import { useId, useRef, useState, type ReactNode } from 'react';

import type { NewDocumentFormState } from '../hooks/useNewDocumentForm';
import { useI18n } from '../i18n/context';
import { TiptapEditor } from './TiptapEditor';

type IndexDocumentFormProps = {
  form: NewDocumentFormState;
};

const labelCls =
  'mb-1 block text-[10px] font-semibold uppercase tracking-[0.12em] text-inkly-muted';
const inputCls =
  'w-full rounded-md border border-inkly-border/90 bg-white px-2.5 py-1.5 text-sm text-inkly-ink shadow-sm outline-none transition placeholder:text-inkly-faint focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent/25';
const textareaCls =
  'w-full rounded-md border border-inkly-border/90 bg-white px-2.5 py-1.5 font-mono text-[13px] leading-snug text-inkly-ink shadow-sm outline-none transition placeholder:text-inkly-faint focus:border-inkly-accent focus:ring-1 focus:ring-inkly-accent/25';

function FormSection({ children }: { children: ReactNode }) {
  return (
    <section className="rounded-lg border border-inkly-line/40 bg-inkly-paper-warm/20 p-3 shadow-sm ring-1 ring-white/40">
      <div className="space-y-3">{children}</div>
    </section>
  );
}

function UploadIcon() {
  return (
    <svg
      width="32"
      height="32"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="text-inkly-faint"
    >
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="17 8 12 3 7 8" />
      <line x1="12" y1="3" x2="12" y2="15" />
    </svg>
  );
}

function PenIcon({ className }: { className?: string }) {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
    >
      <path d="M12 20h9" />
      <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
    </svg>
  );
}

function TagsInput({
  value,
  onChange,
}: {
  value: string;
  onChange: (v: string) => void;
}) {
  const { t } = useI18n();
  const [draft, setDraft] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  const tags = value
    .split(',')
    .map((t) => t.trim())
    .filter(Boolean);

  function commit(text: string) {
    const trimmed = text.trim();
    if (!trimmed) return;
    const next = [...tags, trimmed];
    onChange(next.join(', '));
    setDraft('');
  }

  function removeAt(idx: number) {
    const next = tags.filter((_, i) => i !== idx);
    onChange(next.join(', '));
  }

  return (
    <div
      className="flex min-h-[2.125rem] flex-wrap items-center gap-1.5 rounded-md border border-inkly-border/90 bg-white px-2 py-1.5 shadow-sm transition focus-within:border-inkly-accent focus-within:ring-1 focus-within:ring-inkly-accent/25"
      onClick={() => inputRef.current?.focus()}
    >
      {tags.map((tag, i) => (
        <span
          key={`${tag}-${i}`}
          className="inline-flex items-center gap-1 rounded bg-inkly-sidebar px-1.5 py-0.5 text-xs font-medium text-inkly-ink"
        >
          {tag}
          <button
            type="button"
            className="ml-0.5 rounded-sm text-inkly-muted transition hover:text-inkly-ink"
            onClick={(e) => {
              e.stopPropagation();
              removeAt(i);
            }}
            aria-label={`Remove ${tag}`}
          >
            <svg
              width="10"
              height="10"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="3"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M18 6 6 18M6 6l12 12" />
            </svg>
          </button>
        </span>
      ))}
      <input
        ref={inputRef}
        className="min-w-[4rem] flex-1 border-none bg-transparent text-sm text-inkly-ink outline-none placeholder:text-inkly-faint"
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ',') {
            e.preventDefault();
            commit(draft);
          } else if (e.key === 'Backspace' && draft === '' && tags.length > 0) {
            removeAt(tags.length - 1);
          }
        }}
        onBlur={() => {
          if (draft.trim()) commit(draft);
        }}
        placeholder={tags.length === 0 ? t('form.tagsPlaceholder') : ''}
      />
    </div>
  );
}

function UploadArea({
  form,
  fileInputId,
}: {
  form: NewDocumentFormState;
  fileInputId: string;
}) {
  const { t } = useI18n();
  const {
    contentFile,
    setContentFile,
    contentFileInputRef,
    clearFileInput,
    isHtmlFileSelected,
    convertHtmlFile,
    converting,
  } = form;

  return (
    <div className="space-y-2">
      <input
        ref={contentFileInputRef}
        id={fileInputId}
        type="file"
        accept=".txt,.md,.markdown,.html,.htm,text/plain,text/markdown,text/html"
        className="sr-only"
        onChange={(e) => {
          const f = e.target.files?.[0];
          setContentFile(f ?? null);
        }}
      />

      {contentFile ? (
        <div className="space-y-2">
          <div className="flex items-center gap-3 rounded-lg border border-inkly-accent/30 bg-inkly-accent/5 px-4 py-3">
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="shrink-0 text-inkly-accent"
            >
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
            </svg>
            <div className="min-w-0 flex-1">
              <p
                className="truncate font-mono text-sm font-medium text-inkly-ink"
                title={contentFile.name}
              >
                {contentFile.name}
              </p>
              <p className="text-[11px] text-inkly-muted">
                {(contentFile.size / 1024).toFixed(1)} KB
              </p>
            </div>
            <button
              type="button"
              className="shrink-0 rounded-md px-2 py-1 text-xs font-medium text-inkly-muted transition hover:bg-inkly-border-soft hover:text-inkly-ink"
              onClick={clearFileInput}
            >
              {t('form.removeFile')}
            </button>
          </div>
          {isHtmlFileSelected && (
            <button
              type="button"
              disabled={converting}
              onClick={convertHtmlFile}
              className="flex w-full items-center justify-center gap-2 rounded-lg border border-inkly-accent/30 bg-white px-3 py-2 text-sm font-medium text-inkly-accent shadow-sm transition hover:bg-inkly-accent/5 disabled:cursor-not-allowed disabled:opacity-60"
            >
              {converting ? (
                <>
                  <svg
                    className="h-4 w-4 animate-spin"
                    viewBox="0 0 24 24"
                    fill="none"
                  >
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                    />
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
                    />
                  </svg>
                  {t('form.converting')}
                </>
              ) : (
                <>
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <polyline points="16 3 21 3 21 8" />
                    <line x1="4" y1="20" x2="21" y2="3" />
                    <polyline points="21 16 21 21 16 21" />
                    <line x1="15" y1="15" x2="21" y2="21" />
                    <line x1="4" y1="4" x2="9" y2="9" />
                  </svg>
                  {t('form.convertToMarkdown')}
                </>
              )}
            </button>
          )}
        </div>
      ) : (
        <label
          htmlFor={fileInputId}
          className="flex cursor-pointer flex-col items-center gap-2 rounded-lg border-2 border-dashed border-inkly-border/80 bg-white/50 px-6 py-8 transition hover:border-inkly-accent/50 hover:bg-inkly-accent/5"
        >
          <UploadIcon />
          <div className="text-center">
            <p className="text-sm font-medium text-inkly-ink-soft">
              {t('form.uploadClick')}
            </p>
            <p className="mt-0.5 text-[11px] text-inkly-faint">
              {t('form.uploadTypes')}
            </p>
          </div>
        </label>
      )}

      <div className="flex items-center justify-center gap-3 py-1">
        <div className="h-px flex-1 bg-inkly-line/50" />
        <span className="text-[10px] font-medium uppercase tracking-wider text-inkly-faint">
          {t('form.or')}
        </span>
        <div className="h-px flex-1 bg-inkly-line/50" />
      </div>

      <button
        type="button"
        onClick={form.switchToEditor}
        className="flex w-full items-center justify-center gap-2 rounded-lg border border-inkly-border/80 bg-white px-4 py-2.5 text-sm font-medium text-inkly-ink-soft shadow-sm transition hover:border-inkly-accent/40 hover:text-inkly-ink"
      >
        <PenIcon />
        {t('form.writeContent')}
      </button>
    </div>
  );
}

function EditorArea({ form }: { form: NewDocumentFormState }) {
  const { t } = useI18n();
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className={`${labelCls} mb-0`}>{t('form.body')}</span>
          {form.convertedFromHtml && (
            <span className="inline-flex items-center gap-1 rounded-full bg-inkly-accent/10 px-2 py-0.5 text-[10px] font-medium text-inkly-accent">
              <svg
                width="10"
                height="10"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
              {t('form.convertedFromHtml')}
            </span>
          )}
        </div>
        <button
          type="button"
          onClick={form.switchToUpload}
          className="rounded-md border border-inkly-border/80 bg-white px-2 py-0.5 text-[10px] font-medium text-inkly-ink-soft shadow-sm transition hover:border-inkly-accent/40 hover:text-inkly-ink"
        >
          {t('form.uploadFileInstead')}
        </button>
      </div>
      <div className="flex min-h-0 flex-1 flex-col">
        <TiptapEditor
          initialContent={form.content}
          onChange={form.setContent}
          placeholder={t('form.editorPlaceholder')}
        />
      </div>
    </div>
  );
}

export function IndexDocumentForm({ form }: IndexDocumentFormProps) {
  const { t } = useI18n();
  const fileInputId = useId();
  const {
    title,
    setTitle,
    contentMode,
    docUrl,
    setDocUrl,
    tagsText,
    setTagsText,
    path,
    setPath,
    note,
    setNote,
    loading,
    formError,
    submit,
    isEditing,
  } = form;

  const isEditor = !isEditing && contentMode === 'editor';

  return (
    <form
      className={`flex flex-col font-inkly-read-ui ${isEditor ? 'min-h-0 flex-1 gap-3' : 'gap-3'}`}
      onSubmit={submit}
    >
      {formError ? (
        <div
          className="rounded-md border border-red-200/90 bg-red-50/90 px-2.5 py-1.5 text-xs leading-snug text-red-800"
          role="alert"
        >
          {formError}
        </div>
      ) : null}

      {isEditor ? (
        <>
          <div>
            <label htmlFor="idx-title" className={labelCls}>
              {t('form.title')} <span className="text-red-400">*</span>
            </label>
            <input
              id="idx-title"
              required
              autoFocus
              className={inputCls}
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder={t('form.pageTitlePlaceholder')}
            />
          </div>

          <EditorArea form={form} />

          <details className="group rounded-lg border border-inkly-line/40 bg-inkly-paper-warm/20 shadow-sm ring-1 ring-white/40">
            <summary className="flex cursor-pointer select-none list-none items-center gap-1.5 px-3 py-2 text-inkly-faint transition hover:text-inkly-muted [&::-webkit-details-marker]:hidden">
              <svg
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                className="transition group-open:rotate-90"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
              <span className="font-inkly-read-ui text-[10px] font-medium tracking-wide">
                {t('form.moreMetadata')}
              </span>
            </summary>
            <div className="space-y-3 border-t border-inkly-line/40 px-3 pb-3 pt-2">
              <div>
                <label htmlFor="idx-path" className={labelCls}>
                  {t('form.path')}
                </label>
                <input
                  id="idx-path"
                  className={`${inputCls} font-mono text-[12px]`}
                  value={path}
                  onChange={(e) => setPath(e.target.value)}
                  placeholder={t('form.pathPlaceholderShort')}
                />
              </div>
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                <div>
                  <label htmlFor="idx-url" className={labelCls}>
                    {t('form.docUrl')}
                  </label>
                  <input
                    id="idx-url"
                    type="url"
                    className={`${inputCls} font-mono text-[12px]`}
                    value={docUrl}
                    onChange={(e) => setDocUrl(e.target.value)}
                    placeholder={t('form.urlPlaceholder')}
                  />
                </div>
                <div>
                  <label className={labelCls}>{t('form.tags')}</label>
                  <TagsInput value={tagsText} onChange={setTagsText} />
                </div>
              </div>
              <div>
                <label className={labelCls}>{t('form.note')}</label>
                <TiptapEditor
                  initialContent={note}
                  onChange={setNote}
                  placeholder={t('form.optionalPlaceholder')}
                  compact
                />
              </div>
            </div>
          </details>
        </>
      ) : isEditing ? (
        <FormSection>
          <div>
            <label htmlFor="idx-title" className={labelCls}>
              {t('form.title')} <span className="text-red-400">*</span>
            </label>
            <input
              id="idx-title"
              required
              autoFocus
              className={inputCls}
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder={t('form.pageTitlePlaceholder')}
            />
          </div>
          <div>
            <label htmlFor="idx-path" className={labelCls}>
              {t('form.path')}
            </label>
            <input
              id="idx-path"
              className={`${inputCls} font-mono text-[12px]`}
              value={path}
              onChange={(e) => setPath(e.target.value)}
              placeholder={t('form.pathPlaceholderShort')}
            />
          </div>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            <div>
              <label htmlFor="idx-url" className={labelCls}>
                {t('form.docUrl')}
              </label>
              <input
                id="idx-url"
                type="url"
                className={`${inputCls} font-mono text-[12px]`}
                value={docUrl}
                onChange={(e) => setDocUrl(e.target.value)}
                placeholder={t('form.urlPlaceholder')}
              />
            </div>
            <div>
              <label className={labelCls}>{t('form.tags')}</label>
              <TagsInput value={tagsText} onChange={setTagsText} />
            </div>
          </div>
          <div>
            <label className={labelCls}>{t('form.note')}</label>
            <TiptapEditor
              initialContent={note}
              onChange={setNote}
              placeholder={t('form.optionalPlaceholder')}
              compact
            />
          </div>
        </FormSection>
      ) : (
        <>
          <FormSection>
            <div>
              <label htmlFor="idx-title" className={labelCls}>
                {t('form.title')} <span className="text-red-400">*</span>
              </label>
              <input
                id="idx-title"
                required
                autoFocus
                className={inputCls}
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder={t('form.pageTitlePlaceholder')}
              />
            </div>
            <UploadArea form={form} fileInputId={fileInputId} />
          </FormSection>

          <FormSection>
            <div>
              <label htmlFor="idx-path" className={labelCls}>
                {t('form.path')}
              </label>
              <input
                id="idx-path"
                className={`${inputCls} font-mono text-[12px]`}
                value={path}
                onChange={(e) => setPath(e.target.value)}
                placeholder={t('form.pathPlaceholderShort')}
              />
              <p className="mt-0.5 text-[10px] leading-tight text-inkly-faint">
                <span className="font-mono text-inkly-muted">/</span>{' '}
                {t('form.pathHintOr')}{' '}
                <span className="font-mono text-inkly-muted">/a/b/</span>
              </p>
            </div>
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
              <div>
                <label htmlFor="idx-url" className={labelCls}>
                  {t('form.docUrl')}
                </label>
                <input
                  id="idx-url"
                  type="url"
                  className={`${inputCls} font-mono text-[12px]`}
                  value={docUrl}
                  onChange={(e) => setDocUrl(e.target.value)}
                  placeholder={t('form.urlPlaceholder')}
                />
              </div>
              <div>
                <label className={labelCls}>{t('form.tags')}</label>
                <TagsInput value={tagsText} onChange={setTagsText} />
              </div>
            </div>
            <div>
              <label className={labelCls}>{t('form.note')}</label>
              <TiptapEditor
                initialContent={note}
                onChange={setNote}
                placeholder={t('form.optionalPlaceholder')}
                compact
              />
            </div>
          </FormSection>
        </>
      )}

      <div className="flex shrink-0 justify-end border-t border-inkly-line/40 pt-2.5">
        <button
          type="submit"
          disabled={loading}
          className="rounded-lg bg-inkly-accent px-4 py-1.5 text-sm font-medium text-white shadow-sm transition hover:bg-inkly-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
        >
          {loading
            ? isEditing
              ? t('form.saving')
              : t('form.indexing')
            : isEditing
              ? t('form.saveChanges')
              : t('form.indexDocument')}
        </button>
      </div>
    </form>
  );
}
