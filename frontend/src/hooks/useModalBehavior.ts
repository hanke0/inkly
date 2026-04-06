import { useEffect, useRef, type RefObject } from 'react';

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
) {
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;

  useEffect(() => {
    if (!open) {
      return;
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') {
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
