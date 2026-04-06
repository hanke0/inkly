import { useEffect, useId, useRef, useState } from 'react';

import { useModalBehavior } from '../hooks/useModalBehavior';
import { useI18n } from '../i18n/context';

type TextUploadEditModalProps = {
  open: boolean;
  onClose: () => void;
  initialText: string;
  onApply: (text: string) => void;
  onResetFromFile: () => Promise<string | null>;
};

export function TextUploadEditModal({
  open,
  onClose,
  initialText,
  onApply,
  onResetFromFile,
}: TextUploadEditModalProps) {
  const { t } = useI18n();
  const titleId = useId();
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [draft, setDraft] = useState(initialText);
  const prevOpenRef = useRef(false);

  useModalBehavior(open, onClose, textareaRef);

  useEffect(() => {
    if (open && !prevOpenRef.current) {
      setDraft(initialText);
    }
    prevOpenRef.current = open;
  }, [open, initialText]);

  if (!open) {
    return null;
  }

  function handleApply() {
    onApply(draft);
    onClose();
  }

  async function handleReset() {
    const next = await onResetFromFile();
    if (next !== null) {
      setDraft(next);
    }
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      className="fixed inset-0 z-[60] flex h-dvh max-h-dvh w-full flex-col overflow-hidden border border-inkly-border bg-inkly-paper"
    >
      <div className="flex shrink-0 items-center justify-between gap-3 border-b border-inkly-line px-4 py-3 sm:px-5">
        <h2
          id={titleId}
          className="font-inkly-read-ui text-base font-semibold text-inkly-ink"
        >
          {t('modal.textUploadEditTitle')}
        </h2>
        <button
          type="button"
          className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-inkly-muted transition-colors hover:bg-inkly-border-soft hover:text-inkly-ink"
          onClick={onClose}
          aria-label={t('modal.closeAria')}
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
      <p className="shrink-0 px-4 pt-2 text-[11px] leading-snug text-inkly-muted sm:px-5">
        {t('form.textUploadEditHint')}
      </p>
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden px-4 py-3 sm:px-5 sm:pb-4">
        <textarea
          ref={textareaRef}
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          spellCheck={false}
          className="min-h-0 w-full flex-1 resize-none rounded-md border border-inkly-border/90 bg-white px-2.5 py-2 font-mono text-[13px] leading-snug text-inkly-ink shadow-sm outline-none transition placeholder:text-inkly-faint focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent/25"
          aria-label={t('modal.textUploadEditTitle')}
        />
      </div>
      <div className="flex shrink-0 flex-wrap items-center justify-end gap-2 border-t border-inkly-line/40 px-4 py-3 sm:px-5">
        <button
          type="button"
          onClick={onClose}
          className="rounded-lg border border-inkly-border/90 bg-white px-3 py-1.5 text-sm font-medium text-inkly-ink-soft shadow-sm transition hover:border-inkly-accent/40 hover:text-inkly-ink"
        >
          {t('form.htmlCleanupCancel')}
        </button>
        <button
          type="button"
          onClick={handleReset}
          className="rounded-lg border border-inkly-border/90 bg-white px-3 py-1.5 text-sm font-medium text-inkly-ink-soft shadow-sm transition hover:border-inkly-accent/40 hover:text-inkly-ink"
        >
          {t('form.htmlCleanupReset')}
        </button>
        <button
          type="button"
          onClick={handleApply}
          className="rounded-lg bg-inkly-accent px-3 py-1.5 text-sm font-medium text-white shadow-sm transition hover:bg-inkly-accent-hover"
        >
          {t('form.htmlCleanupApply')}
        </button>
      </div>
    </div>
  );
}
