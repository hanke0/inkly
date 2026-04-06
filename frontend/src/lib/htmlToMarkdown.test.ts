import { afterEach, describe, expect, it, vi } from 'vitest';

import { htmlToMarkdown, htmlToMarkdownInlineImages } from './htmlToMarkdown';

const BASE64_PNG =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/w8AAgMBAp7xN2kAAAAASUVORK5CYII=';

afterEach(() => {
  vi.restoreAllMocks();
});

describe('htmlToMarkdown', () => {
  it('keeps base64 image src when converting to markdown', () => {
    const html = `<p>hello</p><img src="${BASE64_PNG}" alt="图1 常见数据分布">`;
    const md = htmlToMarkdown(html);

    expect(md).toContain(`![图1 常见数据分布](${BASE64_PNG})`);
  });
});

describe('htmlToMarkdownInlineImages', () => {
  it('does not replace data URI images', async () => {
    const html = `<img src="${BASE64_PNG}" alt="inline">`;
    const md = await htmlToMarkdownInlineImages(html);

    expect(md).toContain(`![inline](${BASE64_PNG})`);
  });

  it('prefers data-sf-original-src over transparent svg placeholder', async () => {
    const placeholder =
      'data:image/svg+xml,%3Csvg%20xmlns=%22http://www.w3.org/2000/svg%22%20width=%22100%22%20height=%22100%22%3E%3Crect%20fill-opacity=%220%22/%3E%3C/svg%3E';
    const original = 'https://example.com/real.png';
    vi.spyOn(globalThis, 'fetch').mockRejectedValue(
      new Error('network denied'),
    );

    const html = `<img src="${placeholder}" data-sf-original-src="${original}" alt="图1">`;
    const md = await htmlToMarkdownInlineImages(html);

    expect(md).toContain(`![图1](${original})`);
    expect(md).not.toContain(placeholder);
  });
});
