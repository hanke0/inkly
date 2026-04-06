import type { ReactNode } from 'react';

import { BrandHeader, type BrandHeaderSearchProps } from './BrandHeader';
import { CatalogSidebar } from './CatalogSidebar';
import type { CatalogResponse } from '../types';

type SidebarLayoutProps = {
  searchHeaderProps: BrandHeaderSearchProps;
  sidebarHeaderExtra?: ReactNode;
  catalog: CatalogResponse | null;
  catalogLoading: boolean;
  onCatalogPathChange: (path: string) => void;
  onNewDocument: () => void;
  mainClassName?: string;
  children: ReactNode;
};

export function SidebarLayout({
  searchHeaderProps,
  sidebarHeaderExtra,
  catalog,
  catalogLoading,
  onCatalogPathChange,
  onNewDocument,
  mainClassName,
  children,
}: SidebarLayoutProps) {
  return (
    <div className="flex h-full min-h-0 w-full max-w-full flex-col bg-inkly-shell text-inkly-ink md:flex-row">
      <aside className="flex max-h-[45%] min-h-0 shrink-0 flex-col border-b border-inkly-line bg-gradient-to-b from-inkly-sidebar to-inkly-sidebar-deep md:max-h-none md:w-[17.5rem] md:border-b-0 md:border-r md:shadow-[inset_-1px_0_0_rgba(196,189,176,0.45)]">
        <div className="relative z-20 shrink-0 border-b border-inkly-line/70 bg-inkly-sidebar/30 px-3 py-3 md:px-4">
          <BrandHeader search={searchHeaderProps} />
          {sidebarHeaderExtra}
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto px-3 py-2.5 md:px-4">
          <CatalogSidebar
            catalog={catalog}
            catalogLoading={catalogLoading}
            onPathChange={onCatalogPathChange}
            onNewDocument={onNewDocument}
          />
        </div>
      </aside>
      <main
        className={
          mainClassName ?? 'flex min-h-0 min-w-0 flex-1 flex-col bg-inkly-paper'
        }
      >
        {children}
      </main>
    </div>
  );
}
