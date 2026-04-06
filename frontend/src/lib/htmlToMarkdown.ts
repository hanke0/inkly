import TurndownService from 'turndown';

const turndown = new TurndownService({
  headingStyle: 'atx',
  codeBlockStyle: 'fenced',
  bulletListMarker: '-',
  emDelimiter: '*',
});

turndown.addRule('strikethrough', {
  filter: ['del', 's'],
  replacement: (content) => `~~${content}~~`,
});

turndown.addRule('strikethroughLegacy', {
  filter: (node) => node.nodeName === 'STRIKE',
  replacement: (content) => `~~${content}~~`,
});

/**
 * Convert an HTML string to Markdown.
 * Strips `<style>`, `<script>`, and `<head>` blocks before conversion.
 */
export function htmlToMarkdown(html: string): string {
  let cleaned = html
    .replace(/<head[\s\S]*?<\/head>/gi, '')
    .replace(/<style[\s\S]*?<\/style>/gi, '')
    .replace(/<script[\s\S]*?<\/script>/gi, '');

  const bodyMatch = cleaned.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
  if (bodyMatch) {
    cleaned = bodyMatch[1];
  }

  return turndown.turndown(cleaned).trim();
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
