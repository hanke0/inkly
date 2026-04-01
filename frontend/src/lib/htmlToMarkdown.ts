import TurndownService from "turndown";

const turndown = new TurndownService({
  headingStyle: "atx",
  codeBlockStyle: "fenced",
  bulletListMarker: "-",
  emDelimiter: "*",
});

turndown.addRule("strikethrough", {
  filter: ["del", "s"],
  replacement: (content) => `~~${content}~~`,
});

turndown.addRule("strikethroughLegacy", {
  filter: (node) => node.nodeName === "STRIKE",
  replacement: (content) => `~~${content}~~`,
});

/**
 * Convert an HTML string to Markdown.
 * Strips `<style>`, `<script>`, and `<head>` blocks before conversion.
 */
export function htmlToMarkdown(html: string): string {
  let cleaned = html
    .replace(/<head[\s\S]*?<\/head>/gi, "")
    .replace(/<style[\s\S]*?<\/style>/gi, "")
    .replace(/<script[\s\S]*?<\/script>/gi, "");

  const bodyMatch = cleaned.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
  if (bodyMatch) {
    cleaned = bodyMatch[1];
  }

  return turndown.turndown(cleaned).trim();
}

const HTML_EXTENSIONS = new Set([".html", ".htm"]);

export function isHtmlFile(file: File): boolean {
  const name = file.name.toLowerCase();
  return HTML_EXTENSIONS.has(name.slice(name.lastIndexOf(".")));
}

export async function readFileAsText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error("Failed to read file"));
    reader.readAsText(file);
  });
}
