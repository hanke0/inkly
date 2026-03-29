import type { NewDocumentFormState } from "../hooks/useNewDocumentForm";

type IndexDocumentFormProps = {
  form: NewDocumentFormState;
};

export function IndexDocumentForm({ form }: IndexDocumentFormProps) {
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
    <form className="space-y-4" onSubmit={submit}>
      {formError ? (
        <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800">{formError}</div>
      ) : null}

      <div>
        <label className="text-[10px] font-semibold uppercase tracking-[0.14em] text-inkly-muted">title</label>
        <input
          autoFocus
          className="mt-1.5 w-full rounded-md border border-inkly-border bg-white px-3 py-2 text-sm text-inkly-ink shadow-sm outline-none focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="Document title"
        />
      </div>

      <div>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <label className="text-[10px] font-semibold uppercase tracking-[0.14em] text-inkly-muted">content</label>
          <input
            ref={contentFileInputRef}
            type="file"
            accept=".txt,.md,.markdown,.html,.htm,text/plain,text/markdown,text/html"
            className="max-w-full text-xs text-inkly-muted file:mr-2 file:rounded file:border-0 file:bg-inkly-sidebar file:px-2 file:py-1 file:text-inkly-ink-soft"
            onChange={(e) => {
              const f = e.target.files?.[0];
              setContentFile(f ?? null);
            }}
          />
        </div>
        {contentFile ? (
          <div className="mt-1 flex flex-wrap items-center gap-2">
            <p className="text-xs text-inkly-muted">Using file: {contentFile.name}</p>
            <button
              type="button"
              className="text-xs text-inkly-link hover:text-inkly-link-hover"
              onClick={clearFileInput}
            >
              Clear file
            </button>
          </div>
        ) : null}
        <textarea
          className="mt-1.5 h-28 w-full rounded-md border border-inkly-border bg-white px-3 py-2 font-mono text-sm text-inkly-ink shadow-sm outline-none focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent disabled:opacity-50"
          value={content}
          onChange={(e) => setContent(e.target.value)}
          disabled={Boolean(contentFile)}
          placeholder="Paste or type document text, or choose a file above"
        />
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        <div>
          <label className="text-[10px] font-semibold uppercase tracking-[0.14em] text-inkly-muted">doc_url</label>
          <input
            className="mt-1.5 w-full rounded-md border border-inkly-border bg-white px-3 py-2 text-sm text-inkly-ink shadow-sm outline-none focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent"
            value={docUrl}
            onChange={(e) => setDocUrl(e.target.value)}
            placeholder="https://example.com/article"
          />
        </div>
        <div>
          <label className="text-[10px] font-semibold uppercase tracking-[0.14em] text-inkly-muted">path</label>
          <input
            className="mt-1.5 w-full rounded-md border border-inkly-border bg-white px-3 py-2 text-sm text-inkly-ink shadow-sm outline-none focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent"
            value={path}
            onChange={(e) => setPath(e.target.value)}
            placeholder="/ or /folder/"
          />
          <p className="mt-1 text-[11px] text-inkly-faint">
            Normalized to <span className="font-mono">/</span> or <span className="font-mono">/segment/.../</span>
          </p>
        </div>
      </div>

      <div>
        <label className="text-[10px] font-semibold uppercase tracking-[0.14em] text-inkly-muted">tags</label>
        <input
          className="mt-1.5 w-full rounded-md border border-inkly-border bg-white px-3 py-2 text-sm text-inkly-ink shadow-sm outline-none focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent"
          value={tagsText}
          onChange={(e) => setTagsText(e.target.value)}
          placeholder="comma, separated, tags"
        />
      </div>

      <div>
        <label className="text-[10px] font-semibold uppercase tracking-[0.14em] text-inkly-muted">note</label>
        <textarea
          className="mt-1.5 h-20 w-full rounded-md border border-inkly-border bg-white px-3 py-2 font-mono text-sm text-inkly-ink shadow-sm outline-none focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent"
          value={note}
          onChange={(e) => setNote(e.target.value)}
          placeholder="Optional note"
        />
      </div>

      <div className="flex flex-wrap items-center gap-3">
        <button
          type="submit"
          disabled={loading}
          className="rounded-md bg-inkly-accent px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-inkly-accent-hover disabled:opacity-50"
        >
          {loading ? "Indexing…" : "Index document"}
        </button>
      </div>
    </form>
  );
}
