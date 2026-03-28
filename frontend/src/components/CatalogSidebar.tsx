import { Link } from "react-router-dom";

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
  docLink: (docId: number, folderPath: string) => string;
};

export function CatalogSidebar({
  catalog,
  catalogLoading,
  catalogErr,
  onPathChange,
  docLink,
}: CatalogSidebarProps) {
  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="border-b border-inkly-line px-1 pb-3">
        <h2 className="text-[11px] font-semibold uppercase tracking-[0.12em] text-inkly-muted">
          Contents
        </h2>
        {catalogLoading ? (
          <p className="mt-2 text-xs text-inkly-faint">Loading…</p>
        ) : null}
      </div>

      {catalogErr ? (
        <div className="mt-3 rounded-md border border-red-200 bg-red-50 px-2 py-2 text-xs text-red-800">
          {catalogErr}
        </div>
      ) : null}

      {catalog ? (
        <div className="mt-3 min-h-0 flex-1 overflow-y-auto pr-1">
          <nav className="flex flex-wrap items-center gap-x-1 gap-y-0.5 text-xs text-inkly-muted">
            {pathBreadcrumbs(catalog.path).map((crumb, i, arr) => (
              <span key={crumb.path} className="flex items-center gap-1">
                {i > 0 ? <span className="text-inkly-faint">/</span> : null}
                <button
                  type="button"
                  className={
                    i === arr.length - 1
                      ? "text-left font-medium text-inkly-ink"
                      : "text-left text-inkly-muted hover:text-inkly-ink-soft"
                  }
                  onClick={() => onPathChange(crumb.path)}
                >
                  {crumb.label}
                </button>
              </span>
            ))}
          </nav>

          <div className="mt-4 text-[10px] font-semibold uppercase tracking-[0.14em] text-inkly-faint">
            Folders
          </div>
          <ul className="mt-2 space-y-0.5 border-l border-inkly-line pl-2.5">
            {catalog.subdirs.length === 0 ? (
              <li className="text-xs text-inkly-faint">—</li>
            ) : (
              catalog.subdirs.map((s) => (
                <li key={s.path}>
                  <button
                    type="button"
                    className="w-full truncate text-left text-sm text-inkly-ink-soft hover:text-inkly-ink"
                    title={s.path}
                    onClick={() => onPathChange(s.path)}
                  >
                    {s.name}
                  </button>
                </li>
              ))
            )}
          </ul>

          <div className="mt-5 text-[10px] font-semibold uppercase tracking-[0.14em] text-inkly-faint">
            Pages
          </div>
          <ul className="mt-2 space-y-1 border-l border-inkly-line pl-2.5">
            {catalog.files.length === 0 ? (
              <li className="text-xs text-inkly-faint">No documents in this folder.</li>
            ) : (
              catalog.files.map((f) => (
                <li key={f.doc_id}>
                  <Link
                    to={docLink(f.doc_id, catalog.path)}
                    className="block truncate text-sm text-inkly-link hover:text-inkly-link-hover"
                    title={f.title}
                  >
                    {f.title}
                  </Link>
                </li>
              ))
            )}
          </ul>
        </div>
      ) : !catalogLoading && !catalogErr ? (
        <p className="mt-3 text-xs text-inkly-faint">No catalog data.</p>
      ) : null}
    </div>
  );
}
