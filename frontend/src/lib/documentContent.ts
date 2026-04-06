import DOMPurify, { type Config } from 'dompurify';
import katex from 'katex';
import { marked, type TokenizerAndRendererExtension } from 'marked';

/**
 * KaTeX + marked integration aligned with [docsify-katex](https://github.com/upupming/docsify-katex):
 * one inline-level extension, `$$…$$` tried before `$…$`, same regexes as the plugin’s bundle.
 * `output: "html"` avoids MathML in the DOM so DOMPurify keeps a predictable span-based tree.
 */
const docsifyStyleKatexExtension: TokenizerAndRendererExtension = {
  name: 'math',
  level: 'inline',
  start(src: string) {
    return src.match(/\$/)?.index;
  },
  tokenizer(src: string) {
    const blockMatch = /^\$\$((\\.|[^\$\\])+)\$\$/.exec(src);
    if (blockMatch) {
      return {
        type: 'math',
        raw: blockMatch[0],
        text: blockMatch[1].trim(),
        mathLevel: 'block' as const,
      };
    }
    const inlineMatch = /^\$((\\.|[^\$\\])+)\$/.exec(src);
    if (inlineMatch) {
      return {
        type: 'math',
        raw: inlineMatch[0],
        text: inlineMatch[1].trim(),
        mathLevel: 'inline' as const,
      };
    }
    return undefined;
  },
  renderer(token) {
    if (token.type !== 'math') {
      return false;
    }
    const mathLevel = (token as { mathLevel?: string }).mathLevel;
    const text = (token as { text?: string }).text;
    if (
      typeof text !== 'string' ||
      (mathLevel !== 'block' && mathLevel !== 'inline')
    ) {
      return false;
    }
    return katex.renderToString(text, {
      throwOnError: false,
      displayMode: mathLevel === 'block',
      output: 'html',
    });
  },
};

marked.use({ extensions: [docsifyStyleKatexExtension] });

/** Max characters of the first line used to decide HTML vs Markdown. */
export const FIRST_LINE_PROBE_MAX = 1024;

export function firstLineProbe(content: string): string {
  const end = content.search(/\r\n|\n|\r/);
  const line = end === -1 ? content : content.slice(0, end);
  return line.slice(0, FIRST_LINE_PROBE_MAX);
}

/**
 * Heuristic: inspect the start of the first line (already capped) for HTML-like structure.
 */
export function looksLikeHtml(probe: string): boolean {
  const s = probe.trimStart();
  if (s.length === 0) {
    return false;
  }
  if (/^<!DOCTYPE\s+html/i.test(s)) {
    return true;
  }
  if (/^<html[\s>/]/i.test(s)) {
    return true;
  }
  if (/^<\?xml/i.test(s)) {
    return true;
  }
  if (/^<!--/.test(s)) {
    return true;
  }
  if (/^<[a-zA-Z][\w:-]*(\s|>|\/|$)/.test(s)) {
    return true;
  }
  return false;
}

function isProbablyFullHtmlDocument(raw: string): boolean {
  const t = raw.trimStart();
  return /^<!DOCTYPE\s+html/i.test(t) || /^<html[\s>/]/i.test(t);
}

/** KaTeX can emit SVG; keep SVG profile so equations survive sanitization. */
const purifyConfig: Config = {
  USE_PROFILES: { html: true, svg: true },
  // Allow Markdown images using inlined `data:image/...` URIs.
  ADD_DATA_URI_TAGS: ['img'],
};

function sanitizeHtml(dirty: string): string {
  return DOMPurify.sanitize(dirty, purifyConfig) as string;
}

function markdownToHtml(src: string): string {
  const normalized = src
    .replace(/^\uFEFF/, '')
    .replace(/\r\n/g, '\n')
    .replace(/\r/g, '\n');
  const out = marked.parse(normalized, {
    async: false,
    gfm: true,
    breaks: true,
  });
  if (typeof out !== 'string') {
    return '';
  }
  return out;
}

export type DocumentBodyRender =
  | { kind: 'markdown'; html: string }
  | { kind: 'html'; srcdoc: string };

const ATTR_URL_NAMES = ['href', 'src', 'xlink:href', 'formaction'] as const;

function removeAllScripts(root: Document | Element | DocumentFragment) {
  root.querySelectorAll('script').forEach((el) => el.remove());
}

function removeScriptsFromDocument(doc: Document) {
  removeAllScripts(doc);
  doc.querySelectorAll('template').forEach((t) => {
    removeAllScripts(t.content);
  });
}

function stripEventHandlersAndDangerousUrls(doc: Document) {
  doc.querySelectorAll('*').forEach((el) => {
    const removeNames: string[] = [];
    for (const a of Array.from(el.attributes)) {
      const lower = a.name.toLowerCase();
      if (lower.startsWith('on')) {
        removeNames.push(a.name);
      }
    }
    for (const n of removeNames) {
      el.removeAttribute(n);
    }
    for (const name of ATTR_URL_NAMES) {
      const v = el.getAttribute(name);
      if (!v) {
        continue;
      }
      const t = v
        .trim()
        .replace(/[\u0000-\u0020]+/g, '')
        .toLowerCase();
      if (t.startsWith('javascript:') || t.startsWith('vbscript:')) {
        el.removeAttribute(name);
      }
    }
  });
}

function extractDoctypeDeclaration(raw: string): string {
  const m = raw.match(/^[\s\r\n]*(<!DOCTYPE[^>]*>)/i);
  return m ? m[1] : '<!DOCTYPE html>';
}

/** Default serif for sandboxed HTML reading pane; author CSS in the doc can override. */
const IFRAME_BODY_SERIF_STYLE =
  "body{font-family:Georgia,ui-serif,Cambria,'Times New Roman',Times,'Liberation Serif','Songti SC','STSong',serif;font-variant-numeric:lining-nums;}";

function ensureIframeReadingBodyFont(doc: Document) {
  const head = doc.head;
  if (!head) {
    return;
  }
  const style = doc.createElement('style');
  style.setAttribute('data-inkly', 'reading-font');
  style.textContent = IFRAME_BODY_SERIF_STYLE;
  head.insertBefore(style, head.firstChild);
}

function escapeForHtmlText(raw: string): string {
  return raw.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/** When the parser fails, show source as plain text (no script/CSS execution). */
function fallbackSrcdocPlain(raw: string): string {
  const body = escapeForHtmlText(raw);
  return `<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Document</title></head><body><pre style="white-space:pre-wrap;word-break:break-word;margin:0;font-family:system-ui,sans-serif">${body}</pre></body></html>`;
}

/**
 * Build `srcdoc` for a sandboxed iframe: no scripts / event handlers, but preserve
 * full document (including &lt;head&gt; styles and &lt;link rel="stylesheet"&gt;) and arbitrary CSS.
 */
export function buildIframeSrcdocNoJs(raw: string): string {
  if (typeof document === 'undefined') {
    return fallbackSrcdocPlain(raw);
  }

  const doc = new DOMParser().parseFromString(raw, 'text/html');
  const parserErr = doc.querySelector('parsererror');
  if (parserErr) {
    return fallbackSrcdocPlain(raw);
  }

  removeScriptsFromDocument(doc);
  stripEventHandlersAndDangerousUrls(doc);
  ensureIframeReadingBodyFont(doc);

  const root = doc.documentElement;
  if (!root) {
    return fallbackSrcdocPlain(raw);
  }

  const doctype = isProbablyFullHtmlDocument(raw)
    ? extractDoctypeDeclaration(raw)
    : '<!DOCTYPE html>';
  return `${doctype}\n${root.outerHTML}`;
}

const INKLY_STYLE_SELECTOR = 'style[data-inkly],style[data-inkly-html-cleanup]';

/** Class toggled in the HTML upload cleanup iframe (removed before upload). */
export const HTML_UPLOAD_CLEANUP_SELECTED_CLASS =
  'inkly-html-upload-cleanup-selected';

/**
 * Serialize a live iframe `Document` after user edits (e.g. removed nodes) for upload.
 * Strips Inkly-injected styles and cleanup selection outlines; preserves doctype from `originalRaw`.
 */
export function serializeIframeHtmlForUpload(
  doc: Document,
  originalRaw: string,
): string {
  doc.querySelectorAll(INKLY_STYLE_SELECTOR).forEach((el) => el.remove());
  doc
    .querySelectorAll(`.${HTML_UPLOAD_CLEANUP_SELECTED_CLASS}`)
    .forEach((el) => el.classList.remove(HTML_UPLOAD_CLEANUP_SELECTED_CLASS));

  const root = doc.documentElement;
  if (!root) {
    return originalRaw;
  }

  const doctype = isProbablyFullHtmlDocument(originalRaw)
    ? extractDoctypeDeclaration(originalRaw)
    : '<!DOCTYPE html>';
  return `${doctype}\n${root.outerHTML}`;
}

function elementDepthBelowBody(el: Element, body: HTMLElement): number {
  let d = 0;
  let n: Element | null = el;
  while (n && n !== body) {
    d += 1;
    n = n.parentElement;
  }
  return d;
}

/**
 * Remove elements under `doc.body` that are not visible to users:
 * `[hidden]`, `input[type="hidden"]`, `display:none`, `visibility:hidden|collapse`,
 * `opacity:0`, and nodes with both width and height equal to 0.
 * Deepest matches are removed first so nested `display:none` subtrees are handled safely.
 */
export function removeNonDisplayedBodyElements(doc: Document): number {
  const body = doc.body;
  const win = doc.defaultView;
  if (!body || !win) {
    return 0;
  }

  const matches = new Set<Element>();
  for (const el of body.querySelectorAll('*')) {
    if (el.hasAttribute('hidden')) {
      matches.add(el);
      continue;
    }
    if (
      el.tagName === 'INPUT' &&
      el.getAttribute('type')?.toLowerCase() === 'hidden'
    ) {
      matches.add(el);
      continue;
    }
    const cs = win.getComputedStyle(el);
    if (cs.display === 'none') {
      matches.add(el);
      continue;
    }
    if (cs.visibility === 'hidden' || cs.visibility === 'collapse') {
      matches.add(el);
      continue;
    }
    if (Number.parseFloat(cs.opacity) === 0) {
      matches.add(el);
      continue;
    }

    const rect = el.getBoundingClientRect();
    const width =
      rect.width > 0 ? rect.width : el.clientWidth || el.scrollWidth;
    const height =
      rect.height > 0 ? rect.height : el.clientHeight || el.scrollHeight;
    if (width === 0 && height === 0) {
      matches.add(el);
    }
  }

  const ordered = [...matches].sort(
    (a, b) => elementDepthBelowBody(b, body) - elementDepthBelowBody(a, body),
  );

  let removed = 0;
  for (const el of ordered) {
    if (!body.contains(el)) {
      continue;
    }
    el.remove();
    removed += 1;
  }
  return removed;
}

/**
 * Markdown → inline sanitized HTML. HTML → `srcdoc` via DOM strip (no JS, full CSS).
 */
export function buildDocumentBodyRender(content: string): DocumentBodyRender {
  if (!content) {
    return { kind: 'markdown', html: '' };
  }
  const probe = firstLineProbe(content);
  if (!looksLikeHtml(probe)) {
    return { kind: 'markdown', html: sanitizeHtml(markdownToHtml(content)) };
  }
  return { kind: 'html', srcdoc: buildIframeSrcdocNoJs(content) };
}

/**
 * Short Markdown fields (summary, note): GFM + KaTeX, then DOMPurify.
 * Not for full HTML documents (no `looksLikeHtml` branch).
 */
export function renderMarkdownSnippetToSafeHtml(src: string): string {
  if (src.replace(/^\uFEFF/, '').trim() === '') {
    return '';
  }
  return sanitizeHtml(markdownToHtml(src));
}
