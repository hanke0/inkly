import TurndownService from 'turndown';
import { gfm } from 'turndown-plugin-gfm';

const turndown = new TurndownService({
  headingStyle: 'atx',
  codeBlockStyle: 'fenced',
  bulletListMarker: '-',
  emDelimiter: '*',
  strongDelimiter: '**',
  linkStyle: 'inlined',
});
turndown.use(gfm);

// Markdown has no standard syntax for sub/sup; keep as inline HTML.
turndown.addRule('subscript', {
  filter: 'sub',
  replacement: (content) => `<sub>${content}</sub>`,
});

turndown.addRule('superscript', {
  filter: 'sup',
  replacement: (content) => `<sup>${content}</sup>`,
});

/**
 * Convert an HTML string to Markdown.
 * Strips `<style>`, `<script>`, and `<head>` blocks before conversion.
 */
export function htmlToMarkdown(html: string): string {
  if (typeof DOMParser !== 'undefined') {
    const doc = new DOMParser().parseFromString(html, 'text/html');
    doc.querySelectorAll('script,style').forEach((el) => el.remove());
    const cleaned = doc.body
      ? doc.body.innerHTML
      : doc.documentElement.outerHTML;
    return turndown.turndown(cleaned).trim();
  }

  // Fallback for non-browser environments.
  let cleaned = html
    .replace(/<head[\s\S]*?<\/head>/gi, '')
    .replace(/<style[\s\S]*?<\/style>/gi, '')
    .replace(/<script[\s\S]*?<\/script>/gi, '');
  const bodyMatch = cleaned.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
  if (bodyMatch) cleaned = bodyMatch[1];
  return turndown.turndown(cleaned).trim();
}

async function blobToDataUrl(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () =>
      reject(new Error('Failed to convert blob to data URL'));
    reader.readAsDataURL(blob);
  });
}

function isFetchableImageSrc(src: string): boolean {
  const s = src.trim();
  if (s === '') return false;
  if (s.startsWith('data:')) return false;
  if (s.startsWith('#')) return false;
  if (s.startsWith('javascript:')) return false;
  return true;
}

function isLikelyTransparentSvgPlaceholderSrc(src: string): boolean {
  const s = src.trim().toLowerCase();
  if (!s.startsWith('data:image/svg+xml')) {
    return false;
  }
  // Common blank placeholders: transparent rect svg.
  return (
    s.includes('fill-opacity%3d%220%22') ||
    s.includes('fill-opacity%3d%270%27') ||
    s.includes('fill-opacity=%220%22') ||
    s.includes('fill-opacity=%270%27') ||
    s.includes('fill-opacity="0"') ||
    s.includes("fill-opacity='0'")
  );
}

function resolvePreferredImageSrc(img: HTMLImageElement): string | null {
  const src = img.getAttribute('src')?.trim() ?? '';
  const original =
    img.getAttribute('data-sf-original-src')?.trim() ??
    img.getAttribute('data-original')?.trim() ??
    img.getAttribute('data-src')?.trim() ??
    img.getAttribute('data-lazy-src')?.trim() ??
    '';

  if (original !== '') {
    if (src === '' || isLikelyTransparentSvgPlaceholderSrc(src)) {
      return original;
    }
  }
  return src !== '' ? src : null;
}

async function isLikelyPlaceholderSvg(blob: Blob): Promise<boolean> {
  const mime = (blob.type || '').toLowerCase();
  if (!mime.includes('svg')) {
    return false;
  }
  let text = '';
  try {
    text = await blob.text();
  } catch {
    return false;
  }
  const t = text.replace(/\s+/g, ' ').toLowerCase();
  if (!t.includes('<svg')) {
    return false;
  }
  // Conservative heuristic for blank anti-hotlink placeholders:
  // transparent rect only, without meaningful drawing primitives.
  const hasTransparentRect =
    t.includes('fill-opacity="0"') || t.includes("fill-opacity='0'");
  const hasMeaningfulDrawing =
    t.includes('<path') ||
    t.includes('<image') ||
    t.includes('<text') ||
    t.includes('<circle') ||
    t.includes('<ellipse') ||
    t.includes('<polygon') ||
    t.includes('<polyline');
  return hasTransparentRect && !hasMeaningfulDrawing;
}

/**
 * Convert HTML to Markdown and inline reachable `<img src>` as `data:` URLs.
 *
 * Notes:
 * - Keeps original `src` when fetch fails (CORS, 404, invalid URL, etc.).
 * - Does not attempt to resolve local filesystem relative paths from uploaded files.
 */
export async function htmlToMarkdownInlineImages(
  html: string,
): Promise<string> {
  if (typeof DOMParser === 'undefined') {
    return htmlToMarkdown(html);
  }

  const doc = new DOMParser().parseFromString(html, 'text/html');
  const images = Array.from(doc.querySelectorAll<HTMLImageElement>('img[src]'));

  await Promise.all(
    images.map(async (img) => {
      const preferredSrc = resolvePreferredImageSrc(img);
      if (!preferredSrc) {
        return;
      }
      if (preferredSrc !== (img.getAttribute('src') ?? '').trim()) {
        img.setAttribute('src', preferredSrc);
      }
      if (!isFetchableImageSrc(preferredSrc)) {
        return;
      }
      try {
        const res = await fetch(preferredSrc);
        if (!res.ok) {
          return;
        }
        const blob = await res.blob();
        if (await isLikelyPlaceholderSvg(blob)) {
          return;
        }
        const dataUrl = await blobToDataUrl(blob);
        img.setAttribute('src', dataUrl);
      } catch {
        // Keep original source when inlining fails.
      }
    }),
  );

  return htmlToMarkdown(doc.documentElement.outerHTML);
}

const HTML_EXTENSIONS = new Set(['.html', '.htm']);

const TEXT_LIKE_EXTENSIONS = new Set([
  '.txt',
  '.md',
  '.markdown',
  '.html',
  '.htm',
]);

export function isHtmlFile(file: File): boolean {
  const name = file.name.toLowerCase();
  return HTML_EXTENSIONS.has(name.slice(name.lastIndexOf('.')));
}

/** UTF-8 text uploads we load into the draft buffer (HTML + plain / Markdown). */
export function isTextLikeUploadFile(file: File): boolean {
  const name = file.name.toLowerCase();
  const dot = name.lastIndexOf('.');
  if (dot < 0) {
    return false;
  }
  return TEXT_LIKE_EXTENSIONS.has(name.slice(dot));
}

/** MIME for synthetic `File` when uploading edited draft text. */
export function guessUploadFileMimeType(
  name: string,
  fileType: string,
): string {
  const t = fileType.trim();
  if (t !== '') {
    return t;
  }
  const n = name.toLowerCase();
  if (n.endsWith('.html') || n.endsWith('.htm')) {
    return 'text/html;charset=utf-8';
  }
  if (n.endsWith('.md') || n.endsWith('.markdown')) {
    return 'text/markdown;charset=utf-8';
  }
  return 'text/plain;charset=utf-8';
}

export async function readFileAsText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error('Failed to read file'));
    reader.readAsText(file);
  });
}
