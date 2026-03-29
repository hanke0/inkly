import { useState } from "react";
import { useSearchParams } from "react-router-dom";

import { search } from "./api";
import { BrandHeader, DEFAULT_SEARCH_LIMIT } from "./components/BrandHeader";
import { CatalogSidebar } from "./components/CatalogSidebar";
import { NewDocumentModal } from "./components/NewDocumentModal";
import { SearchResultsDialog } from "./components/SearchResultsDialog";
import { useCatalog } from "./hooks/useCatalog";
import { useNewDocumentForm } from "./hooks/useNewDocumentForm";
import type { SearchQuery, SearchResponse } from "./types";

export default function Dashboard() {
  const [searchParams, setSearchParams] = useSearchParams();

  const [q, setQ] = useState<string>("");
  const [limit, setLimit] = useState<number>(DEFAULT_SEARCH_LIMIT);

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>("");
  const [actionStatus, setActionStatus] = useState<string>("");

  const [searchRes, setSearchRes] = useState<SearchResponse | null>(null);
  const [searchResultsOpen, setSearchResultsOpen] = useState(false);
  const [indexModalOpen, setIndexModalOpen] = useState(false);

  const catalogUrlPath = searchParams.get("path")?.trim() || "/";
  const { catalog, catalogLoading, catalogErr, reloadCatalog } = useCatalog(catalogUrlPath);

  const newDocForm = useNewDocumentForm((res) => {
    void reloadCatalog();
    setIndexModalOpen(false);
    setActionStatus(
      typeof res.doc_id === "number"
        ? `Indexed successfully (doc #${res.doc_id}).`
        : "Indexed successfully.",
    );
  });

  const docLink = (docIdNum: number, folderPath: string) =>
    `/doc/${docIdNum}?path=${encodeURIComponent(folderPath)}`;

  function setCatalogPath(p: string) {
    const next = new URLSearchParams(searchParams);
    next.set("path", p);
    setSearchParams(next);
  }

  function openNewDocumentModal() {
    newDocForm.prepareOpen({ path: catalogUrlPath });
    setIndexModalOpen(true);
  }

  async function runSearch() {
    setError("");
    setSearchRes(null);
    setSearchResultsOpen(false);
    setLoading(true);

    const query: SearchQuery = {
      q: q.trim(),
      limit: Math.max(1, Math.min(50, limit)),
    };

    try {
      const res = await search(query);
      setSearchRes(res);
      setSearchResultsOpen(true);
      setActionStatus("Search complete.");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Search request failed.");
      setActionStatus("");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex h-full min-h-0 w-full max-w-full flex-col bg-inkly-shell text-inkly-ink md:flex-row">
      <aside className="flex max-h-[45%] min-h-0 shrink-0 flex-col border-b border-inkly-line bg-gradient-to-b from-inkly-sidebar to-inkly-sidebar-deep md:max-h-none md:w-[17.5rem] md:border-b-0 md:border-r md:shadow-[inset_-1px_0_0_rgba(196,189,176,0.45)]">
        <div className="relative z-20 shrink-0 border-b border-inkly-line/70 bg-inkly-sidebar/30 px-3 py-3 md:px-4">
          <BrandHeader
            search={{
              q,
              onQChange: setQ,
              limit,
              onLimitChange: setLimit,
              onSearch: () => {
                void runSearch();
              },
              loading,
            }}
          />
          {actionStatus ? (
            <p className="mt-2 rounded-md bg-white/25 px-2 py-1 text-[11px] leading-snug text-inkly-muted">
              {actionStatus}
            </p>
          ) : null}
          {error ? (
            <div className="mt-2 rounded-md border border-red-200/90 bg-red-50/95 px-2 py-1.5 text-[11px] leading-snug text-red-800">
              {error}
            </div>
          ) : null}
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto px-3 py-2.5 md:px-4">
          <CatalogSidebar
            catalog={catalog}
            catalogLoading={catalogLoading}
            catalogErr={catalogErr}
            onPathChange={setCatalogPath}
            docLink={docLink}
            onNewDocument={openNewDocumentModal}
          />
        </div>
      </aside>

      <main className="flex min-h-0 min-w-0 flex-1 flex-col bg-inkly-paper">
        <div className="min-h-0 min-w-0 flex-1 overflow-y-auto px-4 py-5 md:px-8 md:py-6">
          {!error ? (
            <p className="text-sm leading-relaxed text-inkly-muted">
              Pick a page in the library or search above.
            </p>
          ) : null}
        </div>
      </main>

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
