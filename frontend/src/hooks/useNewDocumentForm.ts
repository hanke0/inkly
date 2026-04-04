import { useRef, useState } from "react";

import { indexDocument, indexDocumentUpload } from "../api";
import { useI18n } from "../i18n/context";
import { ensureUtf8File } from "../lib/encoding";
import { htmlToMarkdown, isHtmlFile, readFileAsText } from "../lib/htmlToMarkdown";
import { extractErrorMessage } from "../lib/errors";
import type { DocumentDetailResponse, DocumentIn, IndexResponse } from "../types";

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
  const [contentMode, setContentMode] = useState<"upload" | "editor">("upload");
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [contentFile, setContentFile] = useState<File | null>(null);
  const contentFileInputRef = useRef<HTMLInputElement>(null);
  const [docUrl, setDocUrl] = useState("");
  const [tagsText, setTagsText] = useState("");
  const [path, setPath] = useState("");
  const [note, setNote] = useState("");

  const [converting, setConverting] = useState(false);
  const [convertedFromHtml, setConvertedFromHtml] = useState(false);
  const [loading, setLoading] = useState(false);
  const [formError, setFormError] = useState("");

  const isEditing = editingDocId !== null;

  function clearFileInput() {
    setContentFile(null);
    if (contentFileInputRef.current) {
      contentFileInputRef.current.value = "";
    }
  }

  function switchToEditor() {
    clearFileInput();
    setContentMode("editor");
  }

  function switchToUpload() {
    setConvertedFromHtml(false);
    setContentMode("upload");
  }

  const isHtmlFileSelected = contentFile != null && isHtmlFile(contentFile);

  async function convertHtmlFile() {
    if (!contentFile || !isHtmlFile(contentFile)) return;
    setConverting(true);
    setFormError("");
    try {
      const raw = await readFileAsText(contentFile);
      const md = htmlToMarkdown(raw);
      setContent(md);
      setConvertedFromHtml(true);
      clearFileInput();
      setContentMode("editor");
    } catch {
      setFormError(t("form.convertHtmlFailed"));
    } finally {
      setConverting(false);
    }
  }

  /** Call when opening the modal for a new document; optionally scope `path` to the current catalog folder. */
  function prepareOpen(options?: { path?: string }) {
    setFormError("");
    setEditingDocId(null);
    setContentMode("upload");
    setTitle("");
    setContent("");
    clearFileInput();
    setConvertedFromHtml(false);
    setConverting(false);
    setDocUrl("");
    setTagsText("");
    setNote("");
    if (options?.path !== undefined) {
      setPath(options.path);
    }
  }

  function prepareEdit(d: DocumentDetailResponse) {
    setFormError("");
    setEditingDocId(d.doc_id);
    setContentMode("upload");
    setTitle(d.title);
    setContent("");
    clearFileInput();
    setConvertedFromHtml(false);
    setConverting(false);
    setDocUrl(d.doc_url);
    setTagsText(d.tags.join(", "));
    setPath(d.path);
    setNote(d.note);
  }

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setFormError("");
    setLoading(true);

    const tags = tagsText
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);

    const updateId = editingDocId;

    if (!title.trim()) {
      setFormError(t("form.titleRequired"));
      setLoading(false);
      return;
    }

    try {
      let res: IndexResponse;
      if (updateId != null) {
        const payload: DocumentIn = {
          doc_id: updateId,
          title: title.trim(),
          doc_url: docUrl.trim(),
          tags,
          path: path.trim(),
          note,
        };
        res = await indexDocument(payload);
      } else if (contentFile) {
        if (contentFile.size === 0) {
          setFormError(t("form.uploadEmpty"));
          setLoading(false);
          return;
        }
        const utf8File = await ensureUtf8File(contentFile);
        const fd = new FormData();
        fd.append("file", utf8File);
        fd.append("title", title.trim());
        fd.append("doc_url", docUrl.trim());
        fd.append("path", path.trim());
        fd.append("note", note);
        fd.append("tags", tagsText);
        res = await indexDocumentUpload(fd);
      } else if (contentMode === "editor") {
        if (!content.trim()) {
          setFormError(t("form.addContentOrUpload"));
          setLoading(false);
          return;
        }
        const payload: DocumentIn = {
          title: title.trim(),
          content,
          doc_url: docUrl.trim(),
          tags,
          path: path.trim(),
          note,
        };
        res = await indexDocument(payload);
      } else {
        setFormError(t("form.uploadOrEditor"));
        setLoading(false);
        return;
      }
      onSuccessRef.current(res, updateId != null ? { updatedDocId: updateId } : {});
    } catch (err) {
      setFormError(extractErrorMessage(err, t("errors.indexFailed")));
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
