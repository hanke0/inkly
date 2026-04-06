import { useEffect, useRef, useState } from 'react';

import { indexDocumentUpload, updateDocument } from '../api';
import { useI18n } from '../i18n/context';
import { ensureUtf8File } from '../lib/encoding';
import {
  guessUploadFileMimeType,
  htmlToMarkdown,
  htmlToMarkdownInlineImages,
  isHtmlFile,
  isTextLikeUploadFile,
  readFileAsText,
} from '../lib/htmlToMarkdown';
import type { DocumentDetailResponse, IndexResponse } from '../types';

export type IndexSuccessContext = {
  updatedDocId?: number;
};

export function useNewDocumentForm(
  onSuccess: (res: IndexResponse, ctx: IndexSuccessContext) => void,
) {
  const { t } = useI18n();
  const onSuccessRef = useRef(onSuccess);
  onSuccessRef.current = onSuccess;
  const [editingDocId, setEditingDocId] = useState<number | null>(null);
  const [contentMode, setContentMode] = useState<'upload' | 'editor'>('upload');
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [contentFile, setContentFile] = useState<File | null>(null);
  const contentFileInputRef = useRef<HTMLInputElement>(null);
  const [docUrl, setDocUrl] = useState('');
  const [tagsText, setTagsText] = useState('');
  const [path, setPath] = useState('');
  const [note, setNote] = useState('');

  const [converting, setConverting] = useState(false);
  const [convertedFromHtml, setConvertedFromHtml] = useState(false);
  const [htmlUploadText, setHtmlUploadText] = useState<string | null>(null);
  const [htmlUploadLoading, setHtmlUploadLoading] = useState(false);
  const [htmlCleanupModalOpen, setHtmlCleanupModalOpen] = useState(false);
  const [textUploadEditModalOpen, setTextUploadEditModalOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [formError, setFormError] = useState('');

  const isEditing = editingDocId !== null;

  useEffect(() => {
    if (!contentFile || !isTextLikeUploadFile(contentFile)) {
      setHtmlUploadText(null);
      setHtmlUploadLoading(false);
      return;
    }
    let cancelled = false;
    setHtmlUploadLoading(true);
    setHtmlUploadText(null);
    readFileAsText(contentFile)
      .then((text) => {
        if (!cancelled) {
          setHtmlUploadText(text);
          setHtmlUploadLoading(false);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setHtmlUploadText(null);
          setHtmlUploadLoading(false);
          setFormError(t('form.htmlReadFailed'));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [contentFile, t]);

  function clearFileInput() {
    setContentFile(null);
    setHtmlUploadText(null);
    setHtmlUploadLoading(false);
    setHtmlCleanupModalOpen(false);
    setTextUploadEditModalOpen(false);
    if (contentFileInputRef.current) {
      contentFileInputRef.current.value = '';
    }
  }

  function switchToEditor() {
    clearFileInput();
    setContentMode('editor');
  }

  function switchToUpload() {
    setConvertedFromHtml(false);
    setContentMode('upload');
  }

  const isHtmlFileSelected = contentFile != null && isHtmlFile(contentFile);

  async function resetHtmlUploadFromFile(): Promise<string | null> {
    if (!contentFile || !isTextLikeUploadFile(contentFile)) {
      return null;
    }
    setFormError('');
    try {
      const raw = await readFileAsText(contentFile);
      setHtmlUploadText(raw);
      return raw;
    } catch {
      setFormError(t('form.htmlReadFailed'));
      return null;
    }
  }

  function openHtmlCleanupModal() {
    setHtmlCleanupModalOpen(true);
  }

  function closeHtmlCleanupModal() {
    setHtmlCleanupModalOpen(false);
  }

  function openTextUploadEditModal() {
    setTextUploadEditModalOpen(true);
  }

  function closeTextUploadEditModal() {
    setTextUploadEditModalOpen(false);
  }

  async function convertEditedHtmlToMarkdown(
    rawHtml: string,
  ): Promise<boolean> {
    setConverting(true);
    setFormError('');
    try {
      const md = await htmlToMarkdownInlineImages(rawHtml);
      setContent(md);
      setConvertedFromHtml(true);
      clearFileInput();
      setContentMode('editor');
      return true;
    } catch {
      setFormError(t('form.convertHtmlFailed'));
      return false;
    } finally {
      setConverting(false);
    }
  }

  async function convertHtmlFile() {
    if (!contentFile || !isHtmlFile(contentFile)) return;
    try {
      const raw =
        htmlUploadText !== null
          ? htmlUploadText
          : await readFileAsText(contentFile);
      await convertEditedHtmlToMarkdown(raw);
    } catch {
      setFormError(t('form.convertHtmlFailed'));
    }
  }

  /** Call when opening the modal for a new document; optionally scope `path` to the current catalog folder. */
  function prepareOpen(options?: { path?: string }) {
    setFormError('');
    setEditingDocId(null);
    setContentMode('upload');
    setTitle('');
    setContent('');
    clearFileInput();
    setConvertedFromHtml(false);
    setConverting(false);
    setDocUrl('');
    setTagsText('');
    setNote('');
    if (options?.path !== undefined) {
      setPath(options.path);
    }
  }

  function prepareEdit(d: DocumentDetailResponse) {
    setFormError('');
    setEditingDocId(d.doc_id);
    setContentMode('upload');
    setTitle(d.title);
    setContent('');
    clearFileInput();
    setConvertedFromHtml(false);
    setConverting(false);
    setDocUrl(d.doc_url);
    setTagsText(d.tags.join(', '));
    setPath(d.path);
    setNote(d.note);
  }

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setFormError('');
    setLoading(true);

    const tags = tagsText
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean);

    const updateId = editingDocId;

    if (!title.trim()) {
      setFormError(t('form.titleRequired'));
      setLoading(false);
      return;
    }

    try {
      let res: IndexResponse;
      if (updateId != null) {
        res = await updateDocument(updateId, {
          title: title.trim(),
          doc_url: docUrl.trim(),
          tags,
          path: path.trim(),
          note,
        });
      } else if (contentFile) {
        let fileForUpload = contentFile;
        if (isTextLikeUploadFile(contentFile)) {
          let raw = htmlUploadText;
          if (raw === null) {
            try {
              raw = await readFileAsText(contentFile);
            } catch {
              setFormError(t('form.htmlReadFailed'));
              setLoading(false);
              return;
            }
          }
          const mime = guessUploadFileMimeType(
            contentFile.name,
            contentFile.type,
          );
          fileForUpload = new File([raw], contentFile.name, {
            type: mime,
          });
        }
        if (fileForUpload.size === 0) {
          setFormError(t('form.uploadEmpty'));
          setLoading(false);
          return;
        }
        const utf8File = await ensureUtf8File(fileForUpload);
        const fd = new FormData();
        fd.append('file', utf8File);
        fd.append('title', title.trim());
        fd.append('doc_url', docUrl.trim());
        fd.append('path', path.trim());
        fd.append('note', note);
        fd.append('tags', tagsText);
        res = await indexDocumentUpload(fd);
      } else if (contentMode === 'editor') {
        if (!content.trim()) {
          setFormError(t('form.addContentOrUpload'));
          setLoading(false);
          return;
        }
        const file = new File([content], 'content.txt', {
          type: 'text/plain;charset=utf-8',
        });
        const fd = new FormData();
        fd.append('file', file);
        fd.append('title', title.trim());
        fd.append('doc_url', docUrl.trim());
        fd.append('path', path.trim());
        fd.append('note', note);
        fd.append('tags', tagsText);
        res = await indexDocumentUpload(fd);
      } else {
        setFormError(t('form.uploadOrEditor'));
        setLoading(false);
        return;
      }
      onSuccessRef.current(
        res,
        updateId != null ? { updatedDocId: updateId } : {},
      );
    } catch {
    } finally {
      setLoading(false);
    }
  }

  return {
    contentMode,
    switchToEditor,
    switchToUpload,
    title,
    setTitle,
    content,
    setContent,
    contentFile,
    setContentFile,
    contentFileInputRef,
    clearFileInput,
    isHtmlFileSelected,
    htmlUploadText,
    setHtmlUploadText,
    htmlUploadLoading,
    htmlCleanupModalOpen,
    openHtmlCleanupModal,
    closeHtmlCleanupModal,
    resetHtmlUploadFromFile,
    textUploadEditModalOpen,
    openTextUploadEditModal,
    closeTextUploadEditModal,
    convertEditedHtmlToMarkdown,
    convertHtmlFile,
    converting,
    convertedFromHtml,
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
    setFormError,
    submit,
    prepareOpen,
    prepareEdit,
    isEditing,
  };
}

export type NewDocumentFormState = ReturnType<typeof useNewDocumentForm>;
