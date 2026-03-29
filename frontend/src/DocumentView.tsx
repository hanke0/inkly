import { useEffect, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";

import { clearStoredCredentials, fetchDocument, search } from "./api";
import { BrandHeader } from "./components/BrandHeader";
import { CatalogSidebar } from "./components/CatalogSidebar";
import { DocumentBody } from "./components/DocumentBody";
import { NewDocumentModal } from "./components/NewDocumentModal";
import { SearchResultsDialog } from "./components/SearchResultsDialog";
import { useCatalog } from "./hooks/useCatalog";
import { useNewDocumentForm } from "./hooks/useNewDocumentForm";
import type { DocumentDetailResponse, SearchQuery, SearchResponse } from "./types";

export default function DocumentView() {
  const { docId: docIdParam } = useParams();
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();

  const returnPath = searchParams.get("path")?.trim() || "/";
  const docId = docIdParam ? Number.parseInt(docIdParam, 10) : NaN;

  const { catalog, catalogLoading, catalogErr, reloadCatalog } = useCatalog(returnPath);

  const [indexModalOpen, setIndexModalOpen] = useState(false);
  const newDocForm = useNewDocumentForm(() => {
    void reloadCatalog();
    setIndexModalOpen(false);
  });

  const [doc, setDoc] = useState<DocumentDetailResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const [q, setQ] = useState("");
  const [limit, setLimit] = useState(10);
  const [searchLoading, setSearchLoading] = useState(false);
  const [searchErr, setSearchErr] = useState("");
  const [searchRes, setSearchRes] = useState<SearchResponse | null>(null);
  const [searchResultsOpen, setSearchResultsOpen] = useState(false);

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
          setError(err instanceof Error ? err.message : "Failed to load document.");
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

  function logout() {
    clearStoredCredentials();
    navigate("/login", { replace: true });
  }

  const docLink = (docIdNum: number, folderPath: string) =>
    `/doc/${docIdNum}?path=${encodeURIComponent(folderPath)}`;

  function onCatalogPathChange(p: string) {
    navigate({ pathname: "/", search: `?path=${encodeURIComponent(p)}` });
  }

  function openNewDocumentModal() {
    newDocForm.prepareOpen({ path: returnPath });
    setIndexModalOpen(true);
  }

  async function runSearch() {
    setSearchErr("");
    setSearchRes(null);
    setSearchResultsOpen(false);
    setSearchLoading(true);
    const query: SearchQuery = {
      q: q.trim(),
      limit: Math.max(1, Math.min(50, limit)),
    };
    try {
      const res = await search(query);
      setSearchRes(res);
      setSearchResultsOpen(true);
    } catch (err) {
      setSearchErr(err instanceof Error ? err.message : "Search request failed.");
    } finally {
      setSearchLoading(false);
    }
  }

  return (
    <div className="flex min-h-screen flex-col bg-inkly-shell text-inkly-ink">
      <BrandHeader
        onSignOut={logout}
        onNewDocument={openNewDocumentModal}
        search={{
          q,
          onQChange: setQ,
          limit,
          onLimitChange: setLimit,
          onSearch: () => {
            void runSearch();
          },
          loading: searchLoading,
        }}
      />

      <div className="flex min-h-0 flex-1 flex-col md:flex-row">
        <aside className="flex max-h-[38vh] shrink-0 flex-col overflow-hidden border-b border-inkly-line bg-inkly-sidebar md:max-h-none md:w-72 md:border-b-0 md:border-r">
          <div className="min-h-0 flex-1 overflow-y-auto px-4 py-5 md:px-5">
            <CatalogSidebar
              catalog={catalog}
              catalogLoading={catalogLoading}
              catalogErr={catalogErr}
              onPathChange={onCatalogPathChange}
              docLink={docLink}
            />
          </div>
        </aside>

        <main className="flex min-h-0 min-w-0 flex-1 flex-col bg-inkly-paper">
          <div className="min-w-0 flex-1 overflow-y-auto overflow-x-hidden bg-inkly-paper px-5 py-6 md:px-8">
            {searchErr ? (
              <div className="mb-4 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800">
                {searchErr}
              </div>
            ) : null}

            {error ? (
              <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800">
                {error}
              </div>
            ) : null}

            {loading ? (
              <p className="text-sm text-inkly-muted">Loading…</p>
            ) : doc ? (
              <article className="inkly-reading min-w-0 max-w-full pb-8">
                <h1 className="inkly-reading__title">{doc.title}</h1>
                <div className="mt-2 flex min-w-0 flex-wrap items-center gap-x-1 gap-y-0 text-[11px] leading-tight text-inkly-muted">
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
                {doc.note ? (
                  <details className="mt-3 group">
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
                ) : null}
                <DocumentBody content={doc.content} />
              </article>
            ) : null}
          </div>
        </main>
      </div>

      <SearchResultsDialog
        open={searchResultsOpen}
        onClose={() => setSearchResultsOpen(false)}
        response={searchRes}
        docLink={docLink}
        queryHint={q.trim() || undefined}
      />

      <NewDocumentModal
        open={indexModalOpen}
        onClose={() => setIndexModalOpen(false)}
        form={newDocForm}
      />
    </div>
  );
}
