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

function OptionsGlyph({ className }: { className?: string }) {
  return (
    <svg className={className} width="16" height="16" viewBox="0 0 24 24" aria-hidden>
      <circle cx="12" cy="5" r="1.5" fill="currentColor" />
      <circle cx="12" cy="12" r="1.5" fill="currentColor" />
      <circle cx="12" cy="19" r="1.5" fill="currentColor" />
    </svg>
  );
}

/** Left rail: title and search. */
export function BrandHeader({ search }: BrandHeaderProps) {
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
    <div className="flex flex-col gap-2 font-inkly-read-ui">
      <h1 className="font-serif text-lg font-medium leading-none tracking-tight text-inkly-ink">
        Inkly
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
            <div className="relative z-10 shrink-0" ref={optionsRef}>
              <button
                type="button"
                className="flex h-full min-h-[2rem] items-center border-l border-inkly-border-soft/80 px-2 text-inkly-muted transition-colors hover:bg-inkly-paper-warm/50 hover:text-inkly-ink-soft"
                aria-expanded={optionsOpen}
                aria-haspopup="dialog"
                aria-label="Search options"
                onClick={() => setOptionsOpen((o) => !o)}
              >
                <OptionsGlyph />
              </button>
              {optionsOpen ? (
                <div
                  className="absolute right-0 top-[calc(100%+4px)] z-30 w-40 rounded-lg border border-inkly-border bg-inkly-paper p-2.5 shadow-md"
                  role="dialog"
                  aria-label="Search options"
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
                    onChange={(e) => search.onLimitChange(Number(e.target.value))}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                      }
                    }}
                  />
                </div>
              ) : null}
            </div>
          </div>
        </form>
      ) : null}
    </div>
  );
}
