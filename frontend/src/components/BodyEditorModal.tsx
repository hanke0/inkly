import { useId } from 'react';

import { useModalBehavior } from '../hooks/useModalBehavior';
import { useI18n } from '../i18n/context';
import { TiptapEditor } from './TiptapEditor';

type BodyEditorModalProps = {
  open: boolean;
  onClose: () => void;
  content: string;
  onChange: (value: string) => void;
};

export function BodyEditorModal({
  open,
  onClose,
  content,
  onChange,
}: BodyEditorModalProps) {
  const { t } = useI18n();
  const titleId = useId();

  useModalBehavior(open, onClose);

  if (!open) {
    return null;
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      className="fixed inset-0 z-[60] flex h-dvh max-h-dvh w-full flex-col overflow-x-hidden overflow-y-scroll border border-inkly-border bg-inkly-paper"
    >
      <div className="flex shrink-0 items-center justify-between gap-3 border-b border-inkly-line px-4 py-3 sm:px-5">
        <h2
          id={titleId}
          className="font-inkly-read-ui text-base font-semibold text-inkly-ink"
        >
          {t('modal.fullscreenBodyEditorTitle')}
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
      <div className="flex min-h-0 flex-1 flex-col overflow-x-hidden overflow-y-scroll px-4 py-3 sm:px-5 sm:pb-4">
        <div className="flex min-h-0 flex-1">
          <TiptapEditor
            initialContent={content}
            onChange={onChange}
            placeholder={t('form.editorPlaceholder')}
          />
        </div>
      </div>
      <div className="flex shrink-0 justify-end border-t border-inkly-line/40 px-4 py-3 sm:px-5">
        <button
          type="button"
          onClick={onClose}
          className="rounded-lg bg-inkly-accent px-3 py-1.5 text-sm font-medium text-white shadow-sm transition hover:bg-inkly-accent-hover"
        >
          {t('form.doneEditing')}
        </button>
      </div>
    </div>
  );
}
