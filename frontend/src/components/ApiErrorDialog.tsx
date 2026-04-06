import { useCallback, useEffect, useId, useRef, useState } from 'react';

import { useModalBehavior } from '../hooks/useModalBehavior';
import { useI18n } from '../i18n/context';
import {
  setApiErrorListener,
  type ApiAnnouncedError,
} from '../lib/apiErrorNotify';

export function ApiErrorDialog() {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const [payload, setPayload] = useState<ApiAnnouncedError | null>(null);
  const titleId = useId();
  const closeRef = useRef<HTMLButtonElement>(null);

  const onClose = useCallback(() => {
    setOpen(false);
    setPayload(null);
  }, []);

  useEffect(() => {
    function onErr(detail: ApiAnnouncedError) {
      setPayload(detail);
      setOpen(true);
    }
    setApiErrorListener(onErr);
    return () => setApiErrorListener(null);
  }, []);

  useModalBehavior(open, onClose, closeRef);

  const message =
    payload?.source === 'i18n'
      ? t(payload.key)
      : payload?.source === 'text'
        ? payload.text
        : '';

  if (!open || !payload) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-[60] flex items-start justify-center overflow-y-auto bg-inkly-ink/45 px-3 pb-6 pt-8 sm:px-4 sm:pt-11"
      role="presentation"
      onClick={onClose}
    >
      <div
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="w-full max-w-sm rounded-lg border border-red-400/75 bg-red-50 shadow-xl shadow-red-900/15"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="relative px-3 pb-3 pt-2 sm:px-3.5 sm:pb-3.5 sm:pt-2.5">
          <button
            ref={closeRef}
            type="button"
            className="absolute right-1 top-1 flex h-6 w-6 items-center justify-center rounded text-red-600/90 transition-colors hover:bg-red-200/70 hover:text-red-950"
            onClick={onClose}
            aria-label={t('modal.closeAria')}
          >
            <svg
              width="14"
              height="14"
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
          <h2
            id={titleId}
            className="pr-8 text-[13px] font-semibold leading-snug text-red-900"
          >
            {t('errors.apiErrorTitle')}
          </h2>
          <p className="mt-1.5 text-[13px] leading-snug text-red-800">
            {message}
          </p>
        </div>
      </div>
    </div>
  );
}
