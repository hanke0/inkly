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

  useEffect(() => {
    if (!Number.isFinite(docId) || docId < 1) {
      setError("Invalid document id.");
      setLoading(false);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setError("");
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
              {doc.summary ? (
                <div className="mt-3 shrink-0">
                  <details className="group" open>
                    <summary className="cursor-pointer list-none text-[11px] text-inkly-muted marker:content-none [&::-webkit-details-marker]:hidden hover:text-inkly-ink-soft">
                      <span className="underline decoration-inkly-line decoration-dotted underline-offset-2 group-open:no-underline">
                        Summary
                      </span>
                      <span className="ml-1 text-inkly-faint group-open:hidden">(click to show)</span>
                    </summary>
                    <div className="inkly-reading__note mt-2 border-l-2 border-inkly-accent/40 pl-3">
                      {doc.summary}
                    </div>
                  </details>
                </div>
              ) : null}
              {doc.note ? (
                <div className="mt-3 shrink-0">
                  <details className="group" open={htmlReading || undefined}>
                    <summary className="cursor-pointer list-none text-[11px] text-inkly-muted marker:content-none [&::-webkit-details-marker]:hidden hover:text-inkly-ink-soft">
                      <span className="underline decoration-inkly-line decoration-dotted underline-offset-2 group-open:no-underline">
                        Note
                      </span>
                      <span className="ml-1 text-inkly-faint group-open:hidden">(click to show)</span>
                    </summary>
                    <div className="inkly-reading__note mt-2 border-l-2 border-inkly-line pl-3">
                      {doc.note}
                    </div>
                  </details>
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
