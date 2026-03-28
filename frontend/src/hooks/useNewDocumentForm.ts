import { useRef, useState } from "react";

import { indexDocument, indexDocumentUpload } from "../api";
import type { DocumentIn, IndexResponse } from "../types";

export function useNewDocumentForm(onSuccess: (res: IndexResponse) => void) {
  const onSuccessRef = useRef(onSuccess);
  onSuccessRef.current = onSuccess;
  const [title, setTitle] = useState("Hello");
  const [content, setContent] = useState("This is a test document.");
  const [contentFile, setContentFile] = useState<File | null>(null);
  const contentFileInputRef = useRef<HTMLInputElement>(null);
  const [docUrl, setDocUrl] = useState("https://example.com/doc/1");
  const [tagsText, setTagsText] = useState("test,example");
  const [path, setPath] = useState("/");
  const [note, setNote] = useState("Optional note...");

  const [loading, setLoading] = useState(false);
  const [formError, setFormError] = useState("");

  function clearFileInput() {
    setContentFile(null);
    if (contentFileInputRef.current) {
      contentFileInputRef.current.value = "";
    }
  }

  /** Call when opening the modal; optionally scope `path` to the current catalog folder. */
  function prepareOpen(options?: { path?: string }) {
    setFormError("");
    if (options?.path !== undefined) {
      setPath(options.path);
    }
  }

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setFormError("");
    setLoading(true);

    const tags = tagsText
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);

    try {
      let res: IndexResponse;
      if (contentFile) {
        const fd = new FormData();
        fd.append("file", contentFile);
        fd.append("title", title.trim());
        fd.append("doc_url", docUrl.trim());
        fd.append("path", path.trim());
        fd.append("note", note);
        fd.append("tags", tagsText);
        res = await indexDocumentUpload(fd);
      } else {
        if (!content.trim()) {
          setFormError("Add content in the text area or choose a UTF-8 text file.");
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
      }
      onSuccessRef.current(res);
    } catch (err) {
      setFormError(err instanceof Error ? err.message : "Index request failed.");
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
  };
}

export type NewDocumentFormState = ReturnType<typeof useNewDocumentForm>;
