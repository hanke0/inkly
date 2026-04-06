import { useState } from 'react';
import { useSearchParams } from 'react-router-dom';

import { NewDocumentModal } from './components/NewDocumentModal';
import { SearchResultsDialog } from './components/SearchResultsDialog';
import { SidebarLayout } from './components/SidebarLayout';
import { useCatalog } from './hooks/useCatalog';
import { useNewDocumentForm } from './hooks/useNewDocumentForm';
import { useSearch } from './hooks/useSearch';
import { useI18n } from './i18n/context';

export default function Dashboard() {
  const { t, tf } = useI18n();
  const [searchParams, setSearchParams] = useSearchParams();
  const catalogUrlPath = searchParams.get('path')?.trim() || '/';

  const [actionStatus, setActionStatus] = useState('');
  const [indexModalOpen, setIndexModalOpen] = useState(false);

  const { catalog, catalogLoading, reloadCatalog } = useCatalog(catalogUrlPath);
  const searchState = useSearch(catalogUrlPath);

  const newDocForm = useNewDocumentForm((res, ctx) => {
    void reloadCatalog();
    setIndexModalOpen(false);
    if (ctx.updatedDocId != null) {
      setActionStatus(tf('dash.updatedDoc', { id: ctx.updatedDocId }));
    } else {
      setActionStatus(
        typeof res.doc_id === 'number'
          ? tf('dash.indexedWithId', { id: res.doc_id })
          : t('dash.indexed'),
      );
    }
  });

  function setCatalogPath(p: string) {
    const next = new URLSearchParams(searchParams);
    next.set('path', p);
    setSearchParams(next);
  }

  function openNewDocumentModal() {
    newDocForm.prepareOpen({ path: catalogUrlPath });
    setIndexModalOpen(true);
  }

  return (
    <>
      <SidebarLayout
        searchHeaderProps={searchState.headerProps}
        sidebarHeaderExtra={
          <>
            {actionStatus ? (
              <p className="mt-2 rounded-md bg-white/25 px-2 py-1 text-[11px] leading-snug text-inkly-muted">
                {actionStatus}
              </p>
            ) : null}
            {searchState.error ? (
              <div className="mt-2 rounded-md border border-red-200/90 bg-red-50/95 px-2 py-1.5 text-[11px] leading-snug text-red-800">
                {searchState.error}
              </div>
            ) : null}
          </>
        }
        catalog={catalog}
        catalogLoading={catalogLoading}
        onCatalogPathChange={setCatalogPath}
        onNewDocument={openNewDocumentModal}
      >
        <div className="min-h-0 min-w-0 flex-1 overflow-y-auto px-4 py-5 md:px-8 md:py-6">
          {!searchState.error ? (
            <p className="text-sm leading-relaxed text-inkly-muted">
              {t('dash.pickPage')}
            </p>
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
