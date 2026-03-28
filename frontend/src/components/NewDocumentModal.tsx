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
  const closeRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!open) {
      return;
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") {
        onClose();
      }
    }
    document.addEventListener("keydown", onKey);
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    queueMicrotask(() => closeRef.current?.focus());
    return () => {
      document.removeEventListener("keydown", onKey);
      document.body.style.overflow = prevOverflow;
    };
  }, [open, onClose]);

  if (!open) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto bg-inkly-ink/45 px-4 py-8 sm:py-10"
      role="presentation"
      onClick={onClose}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="my-auto w-full max-w-lg rounded-xl border border-inkly-border bg-inkly-paper shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between gap-3 border-b border-inkly-border px-4 py-3 sm:px-5">
          <h2 id={titleId} className="text-base font-semibold text-inkly-ink">
            New document
          </h2>
          <button
            ref={closeRef}
            type="button"
            className="shrink-0 rounded-md px-2.5 py-1.5 text-sm text-inkly-muted transition-colors hover:bg-inkly-border-soft hover:text-inkly-ink"
            onClick={onClose}
          >
            Close
          </button>
        </div>
        <div className="max-h-[min(78vh,40rem)] overflow-y-auto px-4 py-4 sm:px-5 sm:pb-5">
          <IndexDocumentForm form={form} />
        </div>
      </div>
    </div>
  );
}
