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

  it('converts heading tags into atx markdown headings', () => {
    const html = `
      <h1>总览</h1>
      <h2>背景</h2>
      <h3>方案</h3>
      <h4>细节</h4>
    `;
    const md = htmlToMarkdown(html);

    expect(md).toContain('# 总览');
    expect(md).toContain('## 背景');
    expect(md).toContain('### 方案');
    expect(md).toContain('#### 细节');
  });

  it('converts heading tags with id attributes into markdown headings', () => {
    const html = `<h4 id="脏写">脏写</h4>`;
    const md = htmlToMarkdown(html);

    expect(md).toContain('#### 脏写');
    expect(md).not.toContain('<h4');
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

  it('converts code blocks and inline code correctly', () => {
    const html = `
      <p>示例：<code>const x = 1</code></p>
      <pre><code class="language-js">const add = (a, b) => a + b;</code></pre>
    `;
    const md = htmlToMarkdown(html);

    expect(md).toContain('`const x = 1`');
    expect(md).toContain('```');
    expect(md).toContain('const add = (a, b) => a + b;');
  });

  it('converts links and preserves URL text', () => {
    const html = `<p>查看 <a href="https://example.com/docs">文档</a> 获取详情。</p>`;
    const md = htmlToMarkdown(html);

    expect(md).toContain('[文档](https://example.com/docs)');
  });

  it('converts del and s tags into markdown strikethrough', () => {
    const html = `<p><del>旧方案</del> 与 <s>过时实现</s></p>`;
    const md = htmlToMarkdown(html);

    expect(md).toMatch(/~{1,2}旧方案~{1,2}/);
    expect(md).toMatch(/~{1,2}过时实现~{1,2}/);
  });

  it('converts strong and b tags into markdown bold', () => {
    const html = `<p><strong>重点</strong> 与 <b>加粗词</b></p>`;
    const md = htmlToMarkdown(html);

    expect(md).toContain('**重点**');
    expect(md).toContain('**加粗词**');
  });

  it('converts em and i tags into markdown italic', () => {
    const html = `<p><em>说明</em> 和 <i>倾斜词</i></p>`;
    const md = htmlToMarkdown(html);

    expect(md).toContain('*说明*');
    expect(md).toContain('*倾斜词*');
  });

  it('keeps sub and sup semantics via inline html', () => {
    const html = `<p>H<sub>2</sub>O 与 x<sup>2</sup> + y<sup>2</sup></p>`;
    const md = htmlToMarkdown(html);

    expect(md).toContain('H<sub>2</sub>O');
    expect(md).toContain('x<sup>2</sup>');
    expect(md).toContain('y<sup>2</sup>');
  });

  it('supports nested formatting inside sub and sup', () => {
    const html = `<p><sup><strong>2</strong></sup> 与 <sub><em>n</em></sub></p>`;
    const md = htmlToMarkdown(html);

    expect(md).toContain('<sup>**2**</sup>');
    expect(md).toContain('<sub>*n*</sub>');
  });

  it('converts legacy strike tags into markdown strikethrough', () => {
    const html = `<p>请移除 <strike>弃用字段</strike></p>`;
    const md = htmlToMarkdown(html);

    expect(md).toMatch(/~{1,2}弃用字段~{1,2}/);
    expect(md).not.toContain('<strike>');
  });

  it('converts blockquote content', () => {
    const html = `<blockquote><p>这是一段引用</p><p>第二行</p></blockquote>`;
    const md = htmlToMarkdown(html);

    expect(md).toContain('> 这是一段引用');
    expect(md).toContain('> 第二行');
  });

  it('converts nested unordered and ordered lists', () => {
    const html = `
      <ul>
        <li>
          外层
          <ol>
            <li>内层一</li>
            <li>内层二</li>
          </ol>
        </li>
      </ul>
    `;
    const md = htmlToMarkdown(html);

    expect(md).toContain('-   外层');
    expect(md).toContain('1.  内层一');
    expect(md).toContain('2.  内层二');
  });

  it('converts mixed rich content in one pass', () => {
    const html = `
      <h2>章节</h2>
      <p>介绍 <strong>重点</strong> 与 <em>细节</em>。</p>
      <ul><li>项一</li><li>项二</li></ul>
      <table>
        <tr><th>键</th><th>值</th></tr>
        <tr><td>a</td><td>1</td></tr>
      </table>
    `;
    const md = htmlToMarkdown(html);

    expect(md).toContain('## 章节');
    expect(md).toContain('**重点**');
    expect(md).toContain('*细节*');
    expect(md).toContain('-   项一');
    expect(md).toContain('| 键 | 值 |');
    expect(md).toContain('| a | 1 |');
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
