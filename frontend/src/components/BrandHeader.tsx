import { useEffect, useId, useRef, useState } from "react";

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
  onSignOut: () => void;
  search?: BrandHeaderSearchProps;
  /** Opens the new-document flow (e.g. modal). */
  onNewDocument?: () => void;
};

function SearchGlyph({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      width="18"
      height="18"
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

function OptionsGlyph({ className }: { className?: string }) {
  return (
    <svg className={className} width="18" height="18" viewBox="0 0 24 24" aria-hidden>
      <circle cx="12" cy="5" r="1.75" fill="currentColor" />
      <circle cx="12" cy="12" r="1.75" fill="currentColor" />
      <circle cx="12" cy="19" r="1.75" fill="currentColor" />
    </svg>
  );
}

export function BrandHeader({ onSignOut, search, onNewDocument }: BrandHeaderProps) {
  const inputId = useId();
  const [optionsOpen, setOptionsOpen] = useState(false);
  const optionsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!optionsOpen) {
      return;
    }
    function onPointerDown(e: MouseEvent) {
      if (optionsRef.current && !optionsRef.current.contains(e.target as Node)) {
        setOptionsOpen(false);
      }
    }
    document.addEventListener("mousedown", onPointerDown);
    return () => document.removeEventListener("mousedown", onPointerDown);
  }, [optionsOpen]);

  return (
    <header className="shrink-0 border-b border-inkly-border bg-inkly-toolbar">
      <div className="flex w-full flex-wrap items-center gap-3 px-5 py-3 md:flex-nowrap md:gap-5 md:px-8 md:py-3">
        <div className="min-w-0 flex-1 md:max-w-[14rem] md:flex-none lg:max-w-none">
          <h1 className="font-serif text-2xl font-medium tracking-tight text-inkly-ink">Inkly</h1>
          <p className="mt-0.5 max-w-lg text-sm leading-relaxed text-inkly-muted lg:max-w-md">
            Your personal web archive, self-hosted
          </p>
        </div>

        {search ? (
          <form
            className="order-3 w-full min-w-0 basis-full md:order-none md:max-w-xl md:flex-1 md:basis-0"
            onSubmit={(e) => {
              e.preventDefault();
              search.onSearch();
            }}
            role="search"
          >
            <div className="relative z-0 flex min-w-0 flex-1 items-stretch rounded-lg border border-inkly-border bg-white shadow-sm focus-within:border-inkly-accent focus-within:ring-1 focus-within:ring-inkly-accent">
              <label htmlFor={inputId} className="sr-only">
                Search the archive
              </label>
              <button
                type="submit"
                aria-label="Search"
                disabled={Boolean(search.loading)}
                className="flex shrink-0 items-center border-0 bg-transparent py-2 pl-3 pr-1 text-inkly-muted transition-colors hover:text-inkly-accent disabled:cursor-not-allowed disabled:opacity-50"
              >
                <SearchGlyph />
              </button>
              <input
                id={inputId}
                type="search"
                enterKeyHint="search"
                autoComplete="off"
                disabled={Boolean(search.loading)}
                className="min-w-0 flex-1 border-0 bg-transparent py-2 pl-1 pr-2 text-sm text-inkly-ink outline-none placeholder:text-inkly-faint disabled:opacity-60"
                value={search.q}
                onChange={(e) => search.onQChange(e.target.value)}
                placeholder="Search…"
              />
              <div className="relative z-10 shrink-0" ref={optionsRef}>
                <button
                  type="button"
                  className="flex min-h-[2.5rem] items-center border-l border-inkly-border-soft px-3 text-inkly-muted transition-colors hover:bg-inkly-paper-warm hover:text-inkly-ink-soft"
                  aria-expanded={optionsOpen}
                  aria-haspopup="dialog"
                  aria-label="Search options"
                  onClick={() => setOptionsOpen((o) => !o)}
                >
                  <OptionsGlyph />
                </button>
                {optionsOpen ? (
                  <div
                    className="absolute right-0 top-[calc(100%+6px)] z-30 w-44 rounded-lg border border-inkly-border bg-white p-3 shadow-lg"
                    role="dialog"
                    aria-label="Search options"
                  >
                    <label
                      htmlFor={`${inputId}-limit`}
                      className="block text-[10px] font-semibold uppercase tracking-[0.12em] text-inkly-muted"
                    >
                      Result limit
                    </label>
                    <input
                      id={`${inputId}-limit`}
                      type="number"
                      min={1}
                      max={50}
                      className="mt-1.5 w-full rounded-md border border-inkly-border px-2 py-1.5 text-sm text-inkly-ink outline-none focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent"
                      value={search.limit}
                      onChange={(e) => search.onLimitChange(Number(e.target.value))}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") {
                          e.preventDefault();
                        }
                      }}
                    />
                    <p className="mt-2 text-[11px] leading-snug text-inkly-faint">Press Enter in the search field to run.</p>
                  </div>
                ) : null}
              </div>
            </div>
          </form>
        ) : null}

        <div className="order-2 flex shrink-0 items-center gap-2 md:order-none">
          {onNewDocument ? (
            <button
              type="button"
              onClick={onNewDocument}
              className="rounded-md border border-inkly-border bg-white px-3 py-1.5 text-sm font-medium text-inkly-ink-soft shadow-sm transition-colors hover:bg-inkly-border-soft hover:text-inkly-ink"
            >
              New
            </button>
          ) : null}
          <button
            type="button"
            onClick={onSignOut}
            className="rounded-md px-2 py-1 text-sm text-inkly-muted transition-colors hover:bg-inkly-border-soft hover:text-inkly-ink-soft"
          >
            Sign out
          </button>
        </div>
      </div>
    </header>
  );
}
