import { useEffect, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";

import { deleteDocument, fetchDocument } from "./api";
import { DocumentBody } from "./components/DocumentBody";
import { NewDocumentModal } from "./components/NewDocumentModal";
import { SearchResultsDialog } from "./components/SearchResultsDialog";
import { SidebarLayout } from "./components/SidebarLayout";
import { useCatalog } from "./hooks/useCatalog";
import { useNewDocumentForm } from "./hooks/useNewDocumentForm";
import { useSearch } from "./hooks/useSearch";
import { firstLineProbe, looksLikeHtml } from "./lib/documentContent";
import { extractErrorMessage } from "./lib/errors";
import type { DocumentDetailResponse } from "./types";

export default function DocumentView() {
  const { docId: docIdParam } = useParams();
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();

  const returnPath = searchParams.get("path")?.trim() || "/";
  const docId = docIdParam ? Number.parseInt(docIdParam, 10) : NaN;

  const { catalog, catalogLoading, catalogErr, reloadCatalog } = useCatalog(returnPath);
  const searchState = useSearch(returnPath);

  const [indexModalOpen, setIndexModalOpen] = useState(false);
  const newDocForm = useNewDocumentForm((_, ctx) => {
    void reloadCatalog();
    setIndexModalOpen(false);
    if (ctx.updatedDocId != null && ctx.updatedDocId === docId) {
      void fetchDocument(docId)
        .then((d) => setDoc(d))
        .catch((err) =>
          setError(extractErrorMessage(err, "Failed to refresh document.")),
        );
    }
  });

  const [doc, setDoc] = useState<DocumentDetailResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [openPanel, setOpenPanel] = useState<"summary" | "note" | null>(null);

  useEffect(() => {
    if (!Number.isFinite(docId) || docId < 1) {
      setError("Invalid document id.");
      setLoading(false);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setError("");
    setOpenPanel(null);
    fetchDocument(docId)
      .then((d) => {
        if (!cancelled) {
          setDoc(d);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(extractErrorMessage(err, "Failed to load document."));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [docId]);

  function onCatalogPathChange(p: string) {
    navigate({ pathname: "/", search: `?path=${encodeURIComponent(p)}` });
  }

  function openNewDocumentModal() {
    newDocForm.prepareOpen({ path: returnPath });
    setIndexModalOpen(true);
  }

  function openEditDocumentModal() {
    if (!doc) {
      return;
    }
    newDocForm.prepareEdit(doc);
    setIndexModalOpen(true);
  }

  async function confirmDeleteDocument() {
    if (!doc) {
      return;
    }
    const label = doc.title.trim() || `Document ${doc.doc_id}`;
    if (!window.confirm(`Delete "${label}"? This cannot be undone.`)) {
      return;
    }
    setError("");
    try {
      await deleteDocument(doc.doc_id);
      void reloadCatalog();
      navigate({ pathname: "/", search: `?path=${encodeURIComponent(returnPath)}` });
    } catch (err) {
      setError(extractErrorMessage(err, "Delete failed."));
    }
  }

  const htmlReading = doc != null && looksLikeHtml(firstLineProbe(doc.content));

  return (
    <>
      <SidebarLayout
        searchHeaderProps={searchState.headerProps}
        sidebarHeaderExtra={
          searchState.error ? (
            <div className="mt-2 rounded-md border border-red-200/90 bg-red-50/95 px-2 py-1.5 text-[11px] leading-snug text-red-800">
              {searchState.error}
            </div>
          ) : null
        }
        catalog={catalog}
        catalogLoading={catalogLoading}
        catalogErr={catalogErr}
        onCatalogPathChange={onCatalogPathChange}
        onNewDocument={openNewDocumentModal}
        mainClassName="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden bg-inkly-paper"
      >
        <div
          className={`min-h-0 min-w-0 flex-1 overflow-y-auto overflow-x-hidden bg-inkly-paper px-4 pb-3 pt-4 md:px-8 md:pb-4 md:pt-5 ${
            htmlReading ? "flex min-h-0 flex-col" : ""
          }`}
        >
          {error ? (
            <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800">
              {error}
            </div>
          ) : null}

          {loading ? (
            <p className="text-sm text-inkly-muted">Loading…</p>
          ) : doc ? (
            <article
              className={
                htmlReading
                  ? "inkly-reading flex min-h-full min-w-0 max-w-full flex-col pb-2 md:pb-3"
                  : "inkly-reading min-w-0 max-w-full pb-2 md:pb-3"
              }
            >
              <div className="flex flex-wrap items-start justify-between gap-3">
                <h1 className="inkly-reading__title min-w-0 shrink-0">{doc.title}</h1>
                <div className="flex shrink-0 gap-2">
                  <button
                    type="button"
                    onClick={openEditDocumentModal}
                    className="rounded-md border border-inkly-border/90 bg-white px-2.5 py-1 text-[12px] font-medium text-inkly-ink shadow-sm transition hover:border-inkly-accent/50 hover:bg-inkly-paper-warm/30"
                  >
                    Edit
                  </button>
                  <button
                    type="button"
                    onClick={() => void confirmDeleteDocument()}
                    className="rounded-md border border-red-200/90 bg-white px-2.5 py-1 text-[12px] font-medium text-red-800 shadow-sm transition hover:border-red-300 hover:bg-red-50/80"
                  >
                    Delete
                  </button>
                </div>
              </div>
              <div className="mt-2 flex min-w-0 shrink-0 flex-wrap items-center gap-x-1 gap-y-0 text-[11px] leading-tight text-inkly-muted">
                <span className="min-w-0 truncate font-mono" title={doc.path}>
                  {doc.path}
                </span>
                {doc.tags.length > 0 ? (
                  <>
                    <span className="shrink-0 text-inkly-line" aria-hidden>
                      ·
                    </span>
                    <span className="min-w-0 truncate font-mono" title={doc.tags.join(", ")}>
                      {doc.tags.join(", ")}
                    </span>
                  </>
                ) : null}
                {doc.doc_url ? (
                  <>
                    <span className="shrink-0 text-inkly-line" aria-hidden>
                      ·
                    </span>
                    <a
                      href={doc.doc_url}
                      className="min-w-0 max-w-full truncate font-mono text-inkly-link hover:text-inkly-link-hover hover:underline"
                      title={doc.doc_url}
                      target="_blank"
                      rel="noreferrer"
                    >
                      {doc.doc_url}
                    </a>
                  </>
                ) : null}
              </div>
              {doc.summary || doc.note ? (
                <div className="relative mt-2 shrink-0">
                  <div className="flex items-center gap-1.5">
                    {doc.summary ? (
                      <button
                        type="button"
                        onClick={() => setOpenPanel((p) => (p === "summary" ? null : "summary"))}
                        className={`inline-flex items-center gap-1 rounded-full border px-2 py-[3px] text-[11px] font-medium transition-colors ${
                          openPanel === "summary"
                            ? "border-inkly-accent/30 bg-inkly-accent/10 text-inkly-accent"
                            : "border-inkly-border/60 bg-white/80 text-inkly-muted hover:border-inkly-accent/25 hover:text-inkly-accent/80"
                        }`}
                      >
                        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="shrink-0" aria-hidden>
                          <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
                          <polyline points="14 2 14 8 20 8" />
                        </svg>
                        Summary
                      </button>
                    ) : null}
                    {doc.note ? (
                      <button
                        type="button"
                        onClick={() => setOpenPanel((p) => (p === "note" ? null : "note"))}
                        className={`inline-flex items-center gap-1 rounded-full border px-2 py-[3px] text-[11px] font-medium transition-colors ${
                          openPanel === "note"
                            ? "border-inkly-line/50 bg-inkly-paper-warm text-inkly-ink-soft"
                            : "border-inkly-border/60 bg-white/80 text-inkly-muted hover:border-inkly-line/40 hover:text-inkly-ink-soft"
                        }`}
                      >
                        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="shrink-0" aria-hidden>
                          <path d="M12 20h9" />
                          <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
                        </svg>
                        Note
                      </button>
                    ) : null}
                  </div>
                  {openPanel != null ? (
                    <>
                      <div className="fixed inset-0 z-[9]" onClick={() => setOpenPanel(null)} />
                      <div
                        className={`absolute left-0 right-0 z-10 mt-1.5 max-h-[18rem] overflow-y-auto rounded-lg border px-4 py-3 shadow-lg shadow-inkly-ink/[0.06] ${
                          openPanel === "summary"
                            ? "border-inkly-accent/20 bg-inkly-paper"
                            : "border-inkly-border/50 bg-inkly-paper"
                        }`}
                      >
                        <p
                          className={`mb-1.5 text-[10.5px] font-semibold uppercase tracking-wider ${
                            openPanel === "summary" ? "text-inkly-accent/70" : "text-inkly-muted/60"
                          }`}
                        >
                          {openPanel === "summary" ? "Summary" : "Note"}
                        </p>
                        <div className="inkly-reading__meta">
                          {openPanel === "summary" ? doc.summary : doc.note}
                        </div>
                      </div>
                    </>
                  ) : null}
                </div>
              ) : null}
              <DocumentBody content={doc.content} />
            </article>
          ) : null}
        </div>
      </SidebarLayout>

      <SearchResultsDialog
        open={searchState.resultsOpen}
        onClose={searchState.closeResults}
        response={searchState.results}
        queryHint={searchState.searchSummary}
      />

      <NewDocumentModal
        open={indexModalOpen}
        onClose={() => setIndexModalOpen(false)}
        form={newDocForm}
      />
    </>
  );
}
