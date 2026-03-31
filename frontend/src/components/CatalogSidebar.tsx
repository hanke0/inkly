import { Link, useParams } from "react-router-dom";

import { docLink } from "../lib/docLink";
import type { CatalogResponse } from "../types";

function pathBreadcrumbs(normalizedPath: string): { label: string; path: string }[] {
  const out: { label: string; path: string }[] = [{ label: "Home", path: "/" }];
  if (normalizedPath === "/") {
    return out;
  }
  const inner = normalizedPath.replace(/^\/+|\/+$/g, "");
  const parts = inner.split("/").filter(Boolean);
  let prefix = "";
  for (const p of parts) {
    prefix = `${prefix}/${p}`;
    out.push({ label: p, path: `${prefix}/` });
  }
  return out;
}

type CatalogSidebarProps = {
  catalog: CatalogResponse | null;
  catalogLoading: boolean;
  catalogErr: string;
  onPathChange: (path: string) => void;
  /** Opens the new-document flow (e.g. modal). */
  onNewDocument?: () => void;
};

export function CatalogSidebar({
  catalog,
  catalogLoading,
  catalogErr,
  onPathChange,
  onNewDocument,
}: CatalogSidebarProps) {
  const params = useParams();
  const activeDocParam = params.docId;
  const activeDocId =
    activeDocParam !== undefined ? Number.parseInt(activeDocParam, 10) : Number.NaN;
  const hasActiveDoc = Number.isFinite(activeDocId) && activeDocId >= 1;

  return (
    <div className="flex h-full min-h-0 flex-col font-inkly-read-ui">
      <div
        className="flex items-center gap-2 border-b border-inkly-line/70 pb-2"
        aria-busy={catalogLoading}
      >
        <h2 className="min-w-0 flex-1 truncate text-[10px] font-semibold uppercase tracking-[0.12em] text-inkly-muted">
          Library
        </h2>
        {catalogLoading ? (
          <span className="h-1.5 w-10 shrink-0 animate-pulse rounded-sm bg-inkly-line/50" title="Loading" />
        ) : null}
        {onNewDocument ? (
          <button
            type="button"
            onClick={onNewDocument}
            className="flex h-5 w-5 shrink-0 items-center justify-center rounded text-inkly-muted transition-colors hover:bg-white/50 hover:text-inkly-accent"
            aria-label="Add new page"
            title="New page"
          >
            <svg
              width="11"
              height="11"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.25"
              strokeLinecap="round"
              aria-hidden
            >
              <path d="M12 5v14M5 12h14" />
            </svg>
          </button>
        ) : null}
      </div>

      {catalogErr ? (
        <div className="mt-2 rounded-md border border-red-200/90 bg-red-50/95 px-2 py-1.5 text-[11px] leading-snug text-red-800">
          {catalogErr}
        </div>
      ) : null}

      {catalog ? (
        <div className="mt-2 min-h-0 flex-1 overflow-y-auto">
          <nav className="text-[11px] leading-snug text-inkly-muted" aria-label="Folder path">
            <div className="flex flex-wrap items-center gap-x-0.5 gap-y-0.5">
              {pathBreadcrumbs(catalog.path).map((crumb, i, arr) => (
                <span key={crumb.path} className="flex items-center gap-0.5">
                  {i > 0 ? (
                    <span className="text-inkly-faint/70" aria-hidden>
                      /
                    </span>
                  ) : null}
                  <button
                    type="button"
                    className={
                      i === arr.length - 1
                        ? "rounded px-0.5 py-0 text-left font-medium text-inkly-ink"
                        : "rounded px-0.5 py-0 text-left hover:text-inkly-ink-soft"
                    }
                    onClick={() => onPathChange(crumb.path)}
                  >
                    {crumb.label}
                  </button>
                </span>
              ))}
            </div>
          </nav>

          <p className="mb-0.5 mt-2.5 text-[10px] font-semibold uppercase tracking-[0.12em] text-inkly-faint">
            Folders
          </p>
          <ul className="space-y-0">
            {catalog.subdirs.length === 0 ? (
              <li className="py-1 text-[11px] text-inkly-faint">—</li>
            ) : (
              catalog.subdirs.map((s) => (
                <li key={s.path}>
                  <button
                    type="button"
                    className="w-full truncate rounded px-1 py-0.5 text-left text-xs text-inkly-ink-soft hover:bg-white/40 hover:text-inkly-ink"
                    title={s.path}
                    onClick={() => onPathChange(s.path)}
                  >
                    {s.name}
                  </button>
                </li>
              ))
            )}
          </ul>

          <p className="mb-0.5 mt-2.5 text-[10px] font-semibold uppercase tracking-[0.12em] text-inkly-faint">
            Pages
          </p>
          <ul className="space-y-0">
            {catalog.files.length === 0 ? (
              <li className="py-1 text-[11px] text-inkly-faint">None</li>
            ) : (
              catalog.files.map((f) => {
                const active = hasActiveDoc && f.doc_id === activeDocId;
                return (
                  <li key={f.doc_id}>
                    <Link
                      to={docLink(f.doc_id, catalog.path)}
                      className={
                        active
                          ? "block truncate rounded px-1 py-0.5 text-xs font-medium text-inkly-ink bg-white/55"
                          : "block truncate rounded px-1 py-0.5 text-xs text-inkly-link hover:bg-white/40 hover:text-inkly-link-hover"
                      }
                      title={f.title}
                    >
                      {f.title}
                    </Link>
                  </li>
                );
              })
            )}
          </ul>
        </div>
      ) : !catalogLoading && !catalogErr ? (
        <p className="mt-2 text-[11px] text-inkly-faint">No catalog data.</p>
      ) : null}
    </div>
  );
}
