import { useId, useRef } from "react";
import { Link } from "react-router-dom";

import { useModalBehavior } from "../hooks/useModalBehavior";
import { docLink } from "../lib/docLink";
import type { SearchResponse } from "../types";

type SearchResultsDialogProps = {
  open: boolean;
  onClose: () => void;
  response: SearchResponse | null;
  /** Optional line under the title (e.g. the query string). */
  queryHint?: string;
};

export function SearchResultsDialog({
  open,
  onClose,
  response,
  queryHint,
}: SearchResultsDialogProps) {
  const titleId = useId();
  const closeRef = useRef<HTMLButtonElement>(null);
  useModalBehavior(open, onClose, closeRef);

  if (!open || !response) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto bg-inkly-ink/45 px-4 py-8 sm:py-12"
      role="presentation"
      onClick={onClose}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="my-auto w-full max-w-xl rounded-xl border border-inkly-border bg-inkly-paper shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start justify-between gap-3 border-b border-inkly-border px-4 py-3 sm:px-5">
          <div className="min-w-0 flex-1">
            <h2 id={titleId} className="text-base font-semibold text-inkly-ink">
              Search results
            </h2>
            {queryHint ? (
              <p className="mt-1 truncate text-sm text-inkly-muted" title={queryHint}>
                "{queryHint}"
              </p>
            ) : null}
            <p className="mt-1 text-xs text-inkly-faint">{response.total_hits} hits</p>
          </div>
          <button
            ref={closeRef}
            type="button"
            className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-inkly-muted transition-colors hover:bg-inkly-border-soft hover:text-inkly-ink"
            onClick={onClose}
            aria-label="Close"
          >
            <svg
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
              <path d="M18 6 6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        <ul className="max-h-[36rem] space-y-2 overflow-y-auto px-4 py-3 sm:px-5 sm:py-4">
          {response.results.length === 0 ? (
            <li className="rounded-lg border border-inkly-border bg-inkly-paper-warm px-3 py-6 text-center text-sm text-inkly-muted">
              No matches.
            </li>
          ) : (
            response.results.map((r) => (
              <li
                key={r.doc_id}
                className="rounded-lg border border-inkly-border bg-white/95 p-3 shadow-sm"
              >
                <div className="flex items-start justify-between gap-2">
                  <Link
                    to={docLink(r.doc_id, r.path || "/")}
                    className="min-w-0 flex-1 font-medium text-inkly-link hover:text-inkly-link-hover"
                    onClick={onClose}
                  >
                    {r.title}
                  </Link>
                  <span className="shrink-0 font-mono text-xs text-inkly-faint">{r.score.toFixed(2)}</span>
                </div>
                <p className="mt-2 text-sm leading-relaxed text-inkly-muted">{r.snippet}</p>
                <div className="mt-1.5 flex min-w-0 flex-wrap items-center gap-x-1 text-[11px] leading-tight text-inkly-faint">
                  <span className="min-w-0 truncate font-mono" title={r.path}>
                    {r.path}
                  </span>
                  {r.tags.length > 0 ? (
                    <>
                      <span className="text-inkly-line">·</span>
                      <span className="min-w-0 truncate font-mono" title={r.tags.join(", ")}>
                        {r.tags.join(", ")}
                      </span>
                    </>
                  ) : null}
                  {r.doc_url ? (
                    <>
                      <span className="text-inkly-line">·</span>
                      <span className="min-w-0 truncate font-mono" title={r.doc_url}>
                        {r.doc_url}
                      </span>
                    </>
                  ) : null}
                </div>
              </li>
            ))
          )}
        </ul>
      </div>
    </div>
  );
}
