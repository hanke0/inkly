import { useId, type ReactNode } from "react";

import type { NewDocumentFormState } from "../hooks/useNewDocumentForm";

type IndexDocumentFormProps = {
  form: NewDocumentFormState;
};

const labelCls =
  "mb-1 block text-[10px] font-semibold uppercase tracking-[0.12em] text-inkly-muted";
const inputCls =
  "w-full rounded-md border border-inkly-border/90 bg-white px-2.5 py-1.5 text-sm text-inkly-ink shadow-sm outline-none transition placeholder:text-inkly-faint focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent/25";
const textareaCls =
  "w-full rounded-md border border-inkly-border/90 bg-white px-2.5 py-1.5 font-mono text-[13px] leading-snug text-inkly-ink shadow-sm outline-none transition placeholder:text-inkly-faint focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent/25";

function FormSection({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="rounded-lg border border-inkly-line/40 bg-inkly-paper-warm/20 p-3 shadow-sm ring-1 ring-white/40">
      <h3 className="mb-2 border-b border-inkly-line/40 pb-1 font-inkly-read-ui text-[10px] font-semibold uppercase tracking-[0.14em] text-inkly-faint">
        {title}
      </h3>
      <div className="space-y-3">{children}</div>
    </section>
  );
}

export function IndexDocumentForm({ form }: IndexDocumentFormProps) {
  const fileInputId = useId();
  const {
    title,
    setTitle,
    content,
    setContent,
    contentFile,
    setContentFile,
    contentFileInputRef,
    clearFileInput,
    docUrl,
    setDocUrl,
    tagsText,
    setTagsText,
    path,
    setPath,
    note,
    setNote,
    loading,
    formError,
    submit,
  } = form;

  return (
    <form className="space-y-3 font-inkly-read-ui" onSubmit={submit}>
      {formError ? (
        <div
          className="rounded-md border border-red-200/90 bg-red-50/90 px-2.5 py-1.5 text-xs leading-snug text-red-800"
          role="alert"
        >
          {formError}
        </div>
      ) : null}

      <FormSection title="Content">
        <div>
          <label htmlFor="idx-title" className={labelCls}>
            Title
          </label>
          <input
            id="idx-title"
            autoFocus
            className={inputCls}
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Page title"
          />
        </div>

        <div>
          <div className="mb-1 flex flex-wrap items-center justify-between gap-1.5">
            <label htmlFor="idx-content" className={`${labelCls} mb-0`}>
              Body
            </label>
            <input
              ref={contentFileInputRef}
              id={fileInputId}
              type="file"
              accept=".txt,.md,.markdown,.html,.htm,text/plain,text/markdown,text/html"
              className="sr-only"
              onChange={(e) => {
                const f = e.target.files?.[0];
                setContentFile(f ?? null);
              }}
            />
            <label
              htmlFor={fileInputId}
              className="cursor-pointer rounded-md border border-inkly-border/80 bg-white px-2 py-0.5 text-[10px] font-medium text-inkly-ink-soft shadow-sm transition hover:border-inkly-accent/40"
            >
              Upload…
            </label>
          </div>
          {contentFile ? (
            <div className="mb-1.5 flex flex-wrap items-center gap-1.5 rounded-md bg-white/60 px-2 py-1 text-[11px] text-inkly-muted ring-1 ring-inkly-line/35">
              <span className="min-w-0 truncate font-mono" title={contentFile.name}>
                {contentFile.name}
              </span>
              <button
                type="button"
                className="shrink-0 text-[10px] font-medium text-inkly-link underline decoration-inkly-line decoration-dotted underline-offset-1 hover:text-inkly-link-hover"
                onClick={clearFileInput}
              >
                Remove
              </button>
            </div>
          ) : null}
          <textarea
            id="idx-content"
            className={`${textareaCls} min-h-[5.25rem] resize-y disabled:cursor-not-allowed disabled:bg-inkly-paper-warm/40 disabled:text-inkly-muted`}
            value={content}
            onChange={(e) => setContent(e.target.value)}
            disabled={Boolean(contentFile)}
            placeholder="Markdown, text, or HTML — or upload a file."
          />
        </div>
      </FormSection>

      <FormSection title="Details">
        <div>
          <label htmlFor="idx-path" className={labelCls}>
            Folder path
          </label>
          <input
            id="idx-path"
            className={`${inputCls} font-mono text-[12px]`}
            value={path}
            onChange={(e) => setPath(e.target.value)}
            placeholder="/ or /notes/"
          />
          <p className="mt-0.5 text-[10px] leading-tight text-inkly-faint">
            <span className="font-mono text-inkly-muted">/</span> or{" "}
            <span className="font-mono text-inkly-muted">/a/b/</span>
          </p>
        </div>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <div>
            <label htmlFor="idx-url" className={labelCls}>
              Source URL
            </label>
            <input
              id="idx-url"
              type="url"
              className={`${inputCls} font-mono text-[12px]`}
              value={docUrl}
              onChange={(e) => setDocUrl(e.target.value)}
              placeholder="https://…"
            />
          </div>
          <div>
            <label htmlFor="idx-tags" className={labelCls}>
              Tags
            </label>
            <input
              id="idx-tags"
              className={inputCls}
              value={tagsText}
              onChange={(e) => setTagsText(e.target.value)}
              placeholder="a, b, c"
            />
          </div>
        </div>
        <div>
          <label htmlFor="idx-note" className={labelCls}>
            Note
          </label>
          <textarea
            id="idx-note"
            className={`${textareaCls} min-h-[2.75rem] resize-y`}
            value={note}
            onChange={(e) => setNote(e.target.value)}
            placeholder="Optional"
          />
        </div>
      </FormSection>

      <div className="flex justify-end border-t border-inkly-line/40 pt-2.5">
        <button
          type="submit"
          disabled={loading}
          className="rounded-lg bg-inkly-accent px-4 py-1.5 text-sm font-medium text-white shadow-sm transition hover:bg-inkly-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
        >
          {loading ? "Indexing…" : "Index document"}
        </button>
      </div>
    </form>
  );
}
