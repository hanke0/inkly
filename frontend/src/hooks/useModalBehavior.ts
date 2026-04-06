import { useEffect, useRef, type RefObject } from 'react';

export type ModalBehaviorOptions = {
  /** When false, Escape does not call `onClose` (e.g. a nested dialog is open). */
  closeOnEscape?: boolean;
};

/**
 * Shared modal behavior: dismiss on Escape, lock body scroll while open,
 * and optionally auto-focus an element on open.
 *
 * Uses a ref for `onClose` so callers don't need to memoize the callback
 * (fixes the stale-closure / excessive-rerun bug in SearchResultsDialog).
 */
export function useModalBehavior(
  open: boolean,
  onClose: () => void,
  autoFocusRef?: RefObject<HTMLElement | null>,
  options?: ModalBehaviorOptions,
) {
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;
  const closeOnEscapeRef = useRef(options?.closeOnEscape !== false);
  closeOnEscapeRef.current = options?.closeOnEscape !== false;

  useEffect(() => {
    if (!open) {
      return;
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape' && closeOnEscapeRef.current) {
        onCloseRef.current();
      }
    }
    document.addEventListener('keydown', onKey);
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    if (autoFocusRef?.current) {
      queueMicrotask(() => autoFocusRef.current?.focus());
    }
    return () => {
      document.removeEventListener('keydown', onKey);
      document.body.style.overflow = prevOverflow;
    };
  }, [open, autoFocusRef]);
}
