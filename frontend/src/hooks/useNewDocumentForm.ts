import { useRef, useState } from "react";

import { indexDocument, indexDocumentUpload } from "../api";
import { ensureUtf8File } from "../lib/encoding";
import { extractErrorMessage } from "../lib/errors";
import type { DocumentDetailResponse, DocumentIn, IndexResponse } from "../types";

export type IndexSuccessContext = {
  updatedDocId?: number;
};

export function useNewDocumentForm(
  onSuccess: (res: IndexResponse, ctx: IndexSuccessContext) => void,
) {
  const onSuccessRef = useRef(onSuccess);
  onSuccessRef.current = onSuccess;
  const [editingDocId, setEditingDocId] = useState<number | null>(null);
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [contentFile, setContentFile] = useState<File | null>(null);
  const contentFileInputRef = useRef<HTMLInputElement>(null);
  const [docUrl, setDocUrl] = useState("");
  const [tagsText, setTagsText] = useState("");
  const [path, setPath] = useState("");
  const [note, setNote] = useState("");

  const [loading, setLoading] = useState(false);
  const [formError, setFormError] = useState("");

  const isEditing = editingDocId !== null;

  function clearFileInput() {
    setContentFile(null);
    if (contentFileInputRef.current) {
      contentFileInputRef.current.value = "";
    }
  }

  /** Call when opening the modal for a new document; optionally scope `path` to the current catalog folder. */
  function prepareOpen(options?: { path?: string }) {
    setFormError("");
    setEditingDocId(null);
    setTitle("");
    setContent("");
    clearFileInput();
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
    setTitle(d.title);
    setContent(d.content);
    clearFileInput();
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

    try {
      let res: IndexResponse;
      if (contentFile) {
        const utf8File = await ensureUtf8File(contentFile);
        const fd = new FormData();
        fd.append("file", utf8File);
        fd.append("title", title.trim());
        fd.append("doc_url", docUrl.trim());
        fd.append("path", path.trim());
        fd.append("note", note);
        fd.append("tags", tagsText);
        if (updateId != null) {
          fd.append("doc_id", String(updateId));
        }
        res = await indexDocumentUpload(fd);
      } else {
        if (!content.trim()) {
          setFormError("Add content in the text area or upload a text / HTML file.");
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
        if (updateId != null) {
          payload.doc_id = updateId;
        }
        res = await indexDocument(payload);
      }
      onSuccessRef.current(res, updateId != null ? { updatedDocId: updateId } : {});
    } catch (err) {
      setFormError(extractErrorMessage(err, "Index request failed."));
    } finally {
      setLoading(false);
    }
  }

  return {
    title,
    setTitle,
    content,
    setContent,
    contentFile,
    setContentFile,
    contentFileInputRef,
    clearFileInput,
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
