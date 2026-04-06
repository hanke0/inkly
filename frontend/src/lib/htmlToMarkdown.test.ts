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

  it('converts table into GFM markdown table', () => {
    const html = `
      <table>
        <thead>
          <tr><th>名称</th><th>值</th></tr>
        </thead>
        <tbody>
          <tr><td>alpha</td><td>1</td></tr>
          <tr><td>beta</td><td>2</td></tr>
        </tbody>
      </table>
    `;
    const md = htmlToMarkdown(html);

    expect(md).toContain('| 名称 | 值 |');
    expect(md).toContain('| alpha | 1 |');
    expect(md).toContain('| beta | 2 |');
  });

  it('converts unordered list into markdown bullets', () => {
    const html = `
      <ul>
        <li>第一项</li>
        <li>第二项</li>
        <li><strong>第三项</strong></li>
      </ul>
    `;
    const md = htmlToMarkdown(html);

    expect(md).toContain('-   第一项');
    expect(md).toContain('-   第二项');
    expect(md).toContain('-   **第三项**');
  });

  it('converts unordered list with block children into markdown bullets', () => {
    const html = `
      <ul>
        <li><p>甲</p></li>
        <li><div>乙</div></li>
      </ul>
    `;
    const md = htmlToMarkdown(html);

    expect(md).toContain('-   甲');
    expect(md).toContain('-   乙');
  });

  it('converts ordered list into numbered markdown list', () => {
    const html = `
      <ol>
        <li>第一步</li>
        <li><strong>第二步</strong></li>
      </ol>
    `;
    const md = htmlToMarkdown(html);

    expect(md).toContain('1.  第一步');
    expect(md).toContain('2.  **第二步**');
  });

  it('converts ordered list with start attribute and block children', () => {
    const html = `
      <ol start="3">
        <li><p>甲</p></li>
        <li><div>乙</div></li>
      </ol>
    `;
    const md = htmlToMarkdown(html);

    expect(md).toContain('3.  甲');
    expect(md).toContain('4.  乙');
  });

  it('converts the provided Chinese ordered list snippet correctly', () => {
    const html =
      '<ol><li>每个节点各自产生序列号。</li><li>每个操作上带上时间戳。</li><li>预先分配每个分区负责产生的序列号。</li></ol>';
    const md = htmlToMarkdown(html);

    expect(md).toContain('1.  每个节点各自产生序列号。');
    expect(md).toContain('2.  每个操作上带上时间戳。');
    expect(md).toContain('3.  预先分配每个分区负责产生的序列号。');
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
