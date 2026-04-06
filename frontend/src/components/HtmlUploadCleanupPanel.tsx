import { useCallback, useEffect, useRef, useState } from 'react';

import { useI18n } from '../i18n/context';
import {
  HTML_UPLOAD_CLEANUP_SELECTED_CLASS,
  buildIframeSrcdocNoJs,
  removeNonDisplayedBodyElements,
  serializeIframeHtmlForUpload,
} from '../lib/documentContent';

type HtmlUploadCleanupPanelProps = {
  html: string;
  onHtmlChange: (next: string) => void;
  onReset: () => void | Promise<void>;
  /** `modal`: flatter chrome and taller preview for use inside `HtmlUploadCleanupModal`. */
  variant?: 'default' | 'modal';
};

export function HtmlUploadCleanupPanel({
  html,
  onHtmlChange,
  onReset,
  variant = 'default',
}: HtmlUploadCleanupPanelProps) {
  const { t } = useI18n();
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const detachRef = useRef<(() => void) | null>(null);
  const htmlRef = useRef(html);
  const onHtmlChangeRef = useRef(onHtmlChange);
  const selectedRef = useRef<Element | null>(null);
  const pendingScrollRestoreRef = useRef<{ x: number; y: number } | null>(null);
  const [canDeleteSelection, setCanDeleteSelection] = useState(false);

  htmlRef.current = html;
  onHtmlChangeRef.current = onHtmlChange;

  const tryDeleteSelected = useCallback(() => {
    const iframe = iframeRef.current;
    const doc = iframe?.contentDocument;
    if (!doc) {
      return;
    }
    const sel = selectedRef.current;
    if (!sel || !doc.body.contains(sel)) {
      return;
    }
    sel.remove();
    selectedRef.current = null;
    setCanDeleteSelection(false);
    const win = iframe.contentWindow;
    if (win) {
      pendingScrollRestoreRef.current = {
        x: win.scrollX ?? win.pageXOffset,
        y: win.scrollY ?? win.pageYOffset,
      };
    }
    const next = serializeIframeHtmlForUpload(doc, htmlRef.current);
    onHtmlChangeRef.current(next);
    queueMicrotask(() => {
      iframeRef.current?.focus();
    });
  }, []);

  const tryDeleteRef = useRef(tryDeleteSelected);
  tryDeleteRef.current = tryDeleteSelected;

  const removeSelectedFromLiveDoc = tryDeleteSelected;

  const stripNonDisplayedElements = useCallback(() => {
    const iframe = iframeRef.current;
    const doc = iframe?.contentDocument;
    if (!iframe || !doc?.body) {
      return;
    }
    const win = iframe.contentWindow;
    if (win) {
      pendingScrollRestoreRef.current = {
        x: win.scrollX ?? win.pageXOffset,
        y: win.scrollY ?? win.pageYOffset,
      };
    }
    removeNonDisplayedBodyElements(doc);
    selectedRef.current = null;
    setCanDeleteSelection(false);
    const next = serializeIframeHtmlForUpload(doc, htmlRef.current);
    onHtmlChangeRef.current(next);
    queueMicrotask(() => {
      iframeRef.current?.focus();
    });
  }, []);

  const attachHandlers = useCallback(() => {
    detachRef.current?.();
    detachRef.current = null;

    const iframe = iframeRef.current;
    const doc = iframe?.contentDocument;
    if (!doc?.body) {
      return;
    }

    doc
      .querySelectorAll('style[data-inkly-html-cleanup]')
      .forEach((el) => el.remove());

    const cleanupStyle = doc.createElement('style');
    cleanupStyle.setAttribute('data-inkly-html-cleanup', '1');
    cleanupStyle.textContent = `.${HTML_UPLOAD_CLEANUP_SELECTED_CLASS}{outline:2px solid #4a5c4e!important;outline-offset:2px!important;background-color:rgba(74,92,78,0.09)!important;cursor:default!important;}`;
    doc.head?.appendChild(cleanupStyle);

    const clearSel = () => {
      selectedRef.current?.classList.remove(HTML_UPLOAD_CLEANUP_SELECTED_CLASS);
      selectedRef.current = null;
      setCanDeleteSelection(false);
    };

    const onPointerDown = (e: MouseEvent) => {
      let n: Node = e.target as Node;
      if (n.nodeType === Node.TEXT_NODE) {
        const p = n.parentElement;
        if (!p) {
          return;
        }
        n = p;
      }
      // `instanceof Element` is false for nodes from the iframe's realm; use nodeType.
      if (n.nodeType !== Node.ELEMENT_NODE) {
        return;
      }
      const el = n as Element;
      if (el.closest('a[href], area[href]')) {
        e.preventDefault();
      }
      if (el === doc.body || el === doc.documentElement) {
        clearSel();
        return;
      }
      clearSel();
      selectedRef.current = el;
      el.classList.add(HTML_UPLOAD_CLEANUP_SELECTED_CLASS);
      setCanDeleteSelection(true);
      queueMicrotask(() => {
        iframeRef.current?.focus();
      });
    };

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== 'Delete' && e.key !== 'Backspace') {
        return;
      }
      const sel = selectedRef.current;
      if (!sel || !doc.body.contains(sel)) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      tryDeleteRef.current();
    };

    doc.addEventListener('mousedown', onPointerDown, true);
    doc.addEventListener('keydown', onKeyDown, true);

    detachRef.current = () => {
      doc.removeEventListener('mousedown', onPointerDown, true);
      doc.removeEventListener('keydown', onKeyDown, true);
      clearSel();
    };
  }, []);

  const handleIframeLoad = useCallback(() => {
    attachHandlers();
    const pending = pendingScrollRestoreRef.current;
    if (!pending) {
      return;
    }
    const x = pending.x;
    const y = pending.y;
    pendingScrollRestoreRef.current = null;

    const iframe = iframeRef.current;
    const win = iframe?.contentWindow;
    const doc = iframe?.contentDocument;
    if (!win || !doc?.documentElement) {
      return;
    }

    const applyScroll = () => {
      win.scrollTo(x, y);
      doc.documentElement.scrollLeft = x;
      doc.documentElement.scrollTop = y;
      if (doc.body) {
        doc.body.scrollLeft = x;
        doc.body.scrollTop = y;
      }
    };

    queueMicrotask(applyScroll);
    requestAnimationFrame(applyScroll);
    requestAnimationFrame(() => {
      requestAnimationFrame(applyScroll);
    });
  }, [attachHandlers]);

  useEffect(() => {
    function targetIsEditableField(target: EventTarget | null): boolean {
      if (!target || !(target instanceof HTMLElement)) {
        return false;
      }
      const tag = target.tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') {
        return true;
      }
      if (target.isContentEditable) {
        return true;
      }
      return target.closest('[contenteditable="true"]') != null;
    }

    function onWindowKeyDown(e: KeyboardEvent) {
      if (e.key !== 'Delete' && e.key !== 'Backspace') {
        return;
      }
      if (targetIsEditableField(e.target)) {
        return;
      }
      const iframe = iframeRef.current;
      const doc = iframe?.contentDocument;
      if (!doc?.body) {
        return;
      }
      const sel = selectedRef.current;
      if (!sel || !doc.body.contains(sel)) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      tryDeleteRef.current();
    }

    window.addEventListener('keydown', onWindowKeyDown, true);
    return () => window.removeEventListener('keydown', onWindowKeyDown, true);
  }, []);

  useEffect(() => {
    return () => {
      detachRef.current?.();
      detachRef.current = null;
      selectedRef.current = null;
    };
  }, []);

  const isModal = variant === 'modal';
  const rootCls = isModal
    ? 'flex min-h-0 flex-1 flex-col space-y-2'
    : 'space-y-2 rounded-lg border border-inkly-border/80 bg-white/80 p-2 shadow-sm ring-1 ring-white/40';
  const shellCls = isModal
    ? 'inkly-html-upload-cleanup__shell min-h-0 flex-1 overflow-auto rounded-md border border-inkly-line/60 bg-inkly-paper'
    : 'inkly-html-upload-cleanup__shell max-h-[min(50vh,28rem)] min-h-[12rem] overflow-auto rounded-md border border-inkly-line/60 bg-inkly-paper';
  const iframeCls = isModal
    ? 'block h-full min-h-[12rem] w-full border-0 bg-white'
    : 'h-[min(50vh,28rem)] min-h-[12rem] w-full border-0 bg-white';

  return (
    <div className={rootCls}>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="text-[11px] leading-snug text-inkly-muted">
          {t('form.htmlCleanupHelp')}
        </p>
        <div className="flex flex-wrap items-center gap-1.5">
          <button
            type="button"
            disabled={!canDeleteSelection}
            onClick={removeSelectedFromLiveDoc}
            className="rounded-md border border-inkly-border/90 bg-white px-2 py-1 text-[11px] font-medium text-inkly-ink-soft shadow-sm transition hover:border-inkly-accent/40 hover:text-inkly-ink disabled:cursor-not-allowed disabled:opacity-50"
          >
            {t('form.htmlCleanupDeleteSelected')}
          </button>
          <button
            type="button"
            onClick={stripNonDisplayedElements}
            className="rounded-md border border-inkly-border/90 bg-white px-2 py-1 text-[11px] font-medium text-inkly-ink-soft shadow-sm transition hover:border-inkly-accent/40 hover:text-inkly-ink"
          >
            {t('form.htmlCleanupRemoveNonDisplayed')}
          </button>
          <button
            type="button"
            onClick={onReset}
            className="rounded-md border border-inkly-border/90 bg-white px-2 py-1 text-[11px] font-medium text-inkly-ink-soft shadow-sm transition hover:border-inkly-accent/40 hover:text-inkly-ink"
          >
            {t('form.htmlCleanupReset')}
          </button>
        </div>
      </div>
      <div className={shellCls}>
        <iframe
          ref={iframeRef}
          title={t('form.htmlCleanupIframeTitle')}
          className={iframeCls}
          tabIndex={-1}
          srcDoc={buildIframeSrcdocNoJs(html)}
          sandbox="allow-same-origin"
          referrerPolicy="no-referrer"
          onLoad={handleIframeLoad}
        />
      </div>
    </div>
  );
}
