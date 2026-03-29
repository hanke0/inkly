import { useEffect, useId, useRef } from "react";

import { IndexDocumentForm } from "./IndexDocumentForm";
import type { NewDocumentFormState } from "../hooks/useNewDocumentForm";

type NewDocumentModalProps = {
  open: boolean;
  onClose: () => void;
  form: NewDocumentFormState;
};

export function NewDocumentModal({ open, onClose, form }: NewDocumentModalProps) {
  const titleId = useId();
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;

  useEffect(() => {
    if (!open) {
      return;
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") {
        onCloseRef.current();
      }
    }
    document.addEventListener("keydown", onKey);
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("keydown", onKey);
      document.body.style.overflow = prevOverflow;
    };
  }, [open]);

  if (!open) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto bg-inkly-ink/50 px-3 py-6 backdrop-blur-[2px] sm:py-8"
      role="presentation"
      onClick={onClose}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="my-auto w-full max-w-2xl overflow-hidden rounded-xl border border-inkly-border bg-inkly-paper shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between gap-3 border-b border-inkly-line px-4 py-3 sm:px-5">
          <h2 id={titleId} className="font-inkly-read-ui text-base font-semibold text-inkly-ink">
            New document
          </h2>
          <button
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
        <div className="max-h-[36rem] overflow-y-auto px-4 py-3 sm:px-5 sm:pb-4">
          <IndexDocumentForm form={form} />
        </div>
      </div>
    </div>
  );
}
