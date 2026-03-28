import { useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";

import { clearStoredCredentials, search } from "./api";
import { BrandHeader } from "./components/BrandHeader";
import { CatalogSidebar } from "./components/CatalogSidebar";
import { NewDocumentModal } from "./components/NewDocumentModal";
import { SearchResultsDialog } from "./components/SearchResultsDialog";
import { useCatalog } from "./hooks/useCatalog";
import { useNewDocumentForm } from "./hooks/useNewDocumentForm";
import type { SearchQuery, SearchResponse } from "./types";

export default function Dashboard() {
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();

  const [q, setQ] = useState<string>("");
  const [limit, setLimit] = useState<number>(10);

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>("");
  const [actionStatus, setActionStatus] = useState<string>("");

  const [searchRes, setSearchRes] = useState<SearchResponse | null>(null);
  const [searchResultsOpen, setSearchResultsOpen] = useState(false);
  const [indexModalOpen, setIndexModalOpen] = useState(false);

  const catalogUrlPath = searchParams.get("path")?.trim() || "/";
  const { catalog, catalogLoading, catalogErr, reloadCatalog } = useCatalog(catalogUrlPath);

  const newDocForm = useNewDocumentForm(() => {
    void reloadCatalog();
    setIndexModalOpen(false);
    setActionStatus("Indexed successfully.");
  });

  function logout() {
    clearStoredCredentials();
    navigate("/login", { replace: true });
  }

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
          loading,
        }}
      />

      <div className="flex min-h-0 flex-1 flex-col md:flex-row">
        <aside className="flex max-h-[38vh] shrink-0 flex-col overflow-hidden border-b border-inkly-line bg-inkly-sidebar md:max-h-none md:w-72 md:border-b-0 md:border-r">
          <div className="min-h-0 flex-1 overflow-y-auto px-4 py-5 md:px-5">
            <CatalogSidebar
              catalog={catalog}
              catalogLoading={catalogLoading}
              catalogErr={catalogErr}
              onPathChange={setCatalogPath}
              docLink={docLink}
            />
          </div>
        </aside>

        <main className="flex min-h-0 min-w-0 flex-1 flex-col bg-inkly-paper">
          <div className="shrink-0 border-b border-inkly-border-soft bg-inkly-paper px-5 py-2 md:px-8">
            {actionStatus ? <span className="text-xs text-inkly-muted">{actionStatus}</span> : null}
          </div>

          <div className="flex-1 overflow-y-auto bg-inkly-paper px-5 py-6 md:px-8">
            {error ? (
              <div className="mb-4 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800">
                {error}
              </div>
            ) : null}

            {!error ? (
              <p className="text-sm leading-relaxed text-inkly-muted">
                Open a page from the catalog on the left, or search from the header. Use <span className="font-medium text-inkly-ink-soft">New</span> to add a document.
              </p>
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
