import { useEffect, useId, useRef, useState } from "react";
import { Link } from "react-router-dom";

/** Default search options; keep in sync with initial state in Dashboard / DocumentView. */
export const DEFAULT_SEARCH_LIMIT = 10;

export type BrandHeaderSearchProps = {
  q: string;
  onQChange: (v: string) => void;
  limit: number;
  onLimitChange: (v: number) => void;
  /** Called on Enter in the query field (form submit). */
  onSearch: () => void;
  loading?: boolean;
};

type BrandHeaderProps = {
  search?: BrandHeaderSearchProps;
};

function SearchGlyph({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden
    >
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.3-4.3" />
    </svg>
  );
}

/** Cog / gear (outline); size and stroke via className. */
function SettingsGlyph({ className }: { className?: string }) {
  return (
    <svg
      className={["shrink-0", className].filter(Boolean).join(" ")}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden
    >
      <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
      <circle cx="12" cy="12" r="3" />
    </svg>
  );
}

function isDefaultSearchLimit(limit: number): boolean {
  return Number.isFinite(limit) && limit === DEFAULT_SEARCH_LIMIT;
}

/** Left rail: title and search. */
export function BrandHeader({ search }: BrandHeaderProps) {
  const inputId = useId();
  const [optionsOpen, setOptionsOpen] = useState(false);
  const optionsTriggerRef = useRef<HTMLButtonElement>(null);
  const optionsPanelRef = useRef<HTMLDivElement>(null);

  const settingsDirty = search ? !isDefaultSearchLimit(search.limit) : false;

  useEffect(() => {
    if (!optionsOpen) {
      return;
    }
    function onPointerDown(e: MouseEvent) {
      const t = e.target as Node;
      if (
        optionsTriggerRef.current?.contains(t) ||
        optionsPanelRef.current?.contains(t)
      ) {
        return;
      }
      setOptionsOpen(false);
    }
    document.addEventListener("mousedown", onPointerDown);
    return () => document.removeEventListener("mousedown", onPointerDown);
  }, [optionsOpen]);

  function onLimitFieldChange(raw: string) {
    if (!search) {
      return;
    }
    const trimmed = raw.trim();
    const n = trimmed === "" ? Number.NaN : Number(trimmed);
    search.onLimitChange(Number.isFinite(n) ? n : search.limit);
  }

  function resetSearchSettings() {
    if (!search) {
      return;
    }
    search.onLimitChange(DEFAULT_SEARCH_LIMIT);
  }

  return (
    <div className="flex flex-col gap-2 font-inkly-read-ui">
      <h1 className="m-0 text-lg font-medium leading-none tracking-tight">
        <Link
          to="/"
          className="font-serif text-inkly-ink transition-colors hover:text-inkly-accent focus-visible:rounded-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inkly-accent/35 focus-visible:ring-offset-2 focus-visible:ring-offset-inkly-sidebar"
        >
          Inkly
        </Link>
      </h1>

      {search ? (
        <form
          className="w-full min-w-0"
          onSubmit={(e) => {
            e.preventDefault();
            search.onSearch();
          }}
          role="search"
        >
          {/* Panel sits outside overflow-hidden row so it is not clipped; refs split for outside-click. */}
          <div className="relative w-full min-w-0">
            <div className="relative z-0 flex min-w-0 flex-1 items-stretch overflow-hidden rounded-lg border border-inkly-border/80 bg-inkly-paper/90 focus-within:border-inkly-accent focus-within:ring-1 focus-within:ring-inkly-accent/30">
              <label htmlFor={inputId} className="sr-only">
                Search the archive
              </label>
              <button
                type="submit"
                aria-label="Search"
                disabled={Boolean(search.loading)}
                className="flex shrink-0 items-center border-0 bg-transparent py-1.5 pl-2 pr-0.5 text-inkly-muted transition-colors hover:text-inkly-accent disabled:cursor-not-allowed disabled:opacity-50"
              >
                <SearchGlyph />
              </button>
              <input
                id={inputId}
                type="search"
                enterKeyHint="search"
                autoComplete="off"
                disabled={Boolean(search.loading)}
                className="min-w-0 flex-1 border-0 bg-transparent py-1.5 pl-0.5 pr-1 text-xs text-inkly-ink outline-none placeholder:text-inkly-faint disabled:opacity-60"
                value={search.q}
                onChange={(e) => search.onQChange(e.target.value)}
                placeholder="Search…"
              />
              <div className="relative z-10 flex shrink-0 items-center pr-1.5">
                <button
                  ref={optionsTriggerRef}
                  type="button"
                  className="flex h-full min-h-[2rem] items-center border-0 bg-transparent px-0.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inkly-accent/35 focus-visible:ring-offset-2 focus-visible:ring-offset-inkly-paper/90"
                  aria-expanded={optionsOpen}
                  aria-haspopup="dialog"
                  aria-label={
                    settingsDirty
                      ? "Search settings, non-default result limit"
                      : "Search settings"
                  }
                  title={
                    settingsDirty
                      ? "Search settings (result limit differs from default)"
                      : "Search settings"
                  }
                  onClick={() => setOptionsOpen((o) => !o)}
                >
                  <span
                    className={
                      settingsDirty
                        ? "inline-flex items-center justify-center p-0.5 text-inkly-accent-hover"
                        : "inline-flex items-center justify-center p-0.5 text-inkly-line opacity-[0.72]"
                    }
                  >
                    <SettingsGlyph
                      className={
                        settingsDirty
                          ? "h-5 w-5 stroke-[2.35]"
                          : "h-3.5 w-3.5 stroke-[1.55]"
                      }
                    />
                  </span>
                </button>
              </div>
            </div>
            {optionsOpen ? (
              <div
                ref={optionsPanelRef}
                className="absolute right-0 top-[calc(100%+4px)] z-40 w-44 rounded-lg border border-inkly-border bg-inkly-paper p-2.5 shadow-md"
                role="dialog"
                aria-label="Search settings"
              >
                <label
                  htmlFor={`${inputId}-limit`}
                  className="block text-[10px] font-semibold uppercase tracking-wide text-inkly-muted"
                >
                  Result limit
                </label>
                <input
                  id={`${inputId}-limit`}
                  type="number"
                  min={1}
                  max={50}
                  className="mt-1 w-full rounded border border-inkly-border bg-white px-1.5 py-1 text-sm text-inkly-ink outline-none focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent/25"
                  value={search.limit}
                  onChange={(e) => onLimitFieldChange(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                    }
                  }}
                />
                <button
                  type="button"
                  className="mt-2.5 w-full border-0 bg-transparent p-0 text-left text-[11px] text-inkly-link underline-offset-2 transition-colors hover:text-inkly-link-hover hover:underline disabled:cursor-default disabled:text-inkly-faint disabled:no-underline"
                  disabled={!settingsDirty}
                  onClick={() => {
                    resetSearchSettings();
                  }}
                >
                  Clear settings
                </button>
              </div>
            ) : null}
          </div>
        </form>
      ) : null}
    </div>
  );
}
