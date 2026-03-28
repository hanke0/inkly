import { useEffect, useMemo, useState } from "react";
import { Link, useNavigate, useParams, useSearchParams } from "react-router-dom";

import { clearStoredCredentials, fetchDocument } from "./api";
import type { DocumentDetailResponse } from "./types";

export default function DocumentView() {
  const { docId: docIdParam } = useParams();
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const pageOrigin = useMemo(
    () => (typeof window !== "undefined" ? window.location.origin : ""),
    [],
  );

  const returnPath = searchParams.get("path") ?? "/";
  const docId = docIdParam ? Number.parseInt(docIdParam, 10) : NaN;

  const [doc, setDoc] = useState<DocumentDetailResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    if (!Number.isFinite(docId) || docId < 1) {
      setError("Invalid document id.");
      setLoading(false);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setError("");
    fetchDocument(docId)
      .then((d) => {
        if (!cancelled) {
          setDoc(d);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Failed to load document.");
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [docId]);

  function logout() {
    clearStoredCredentials();
    navigate("/login", { replace: true });
  }

  const backHref = `/?path=${encodeURIComponent(returnPath)}`;

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100">
      <div className="mx-auto max-w-3xl px-4 py-10">
        <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
          <div>
            <div className="text-sm text-zinc-400">Inkly SPA</div>
            <div className="text-2xl font-semibold">Document</div>
            <div className="mt-2 text-sm text-zinc-400">API: same origin ({pageOrigin || "…"})</div>
          </div>
          <div className="flex flex-wrap gap-2">
            <Link
              to={backHref}
              className="rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-1.5 text-sm text-zinc-200 hover:bg-zinc-800"
            >
              Back to catalog
            </Link>
            <button
              type="button"
              onClick={logout}
              className="rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-1.5 text-sm text-zinc-200 hover:bg-zinc-800"
            >
              Sign out
            </button>
          </div>
        </div>

        {error ? (
          <div className="rounded-xl border border-red-900 bg-red-950/30 p-3 text-sm text-red-200">{error}</div>
        ) : null}

        {loading ? (
          <div className="mt-4 text-sm text-zinc-400">Loading…</div>
        ) : doc ? (
          <article className="mt-4 rounded-xl border border-zinc-800 bg-zinc-900 p-4">
            <h1 className="text-xl font-semibold text-zinc-50">{doc.title}</h1>
            <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-zinc-400 font-mono">
              <span>doc_id: {doc.doc_id}</span>
              <span>path: {doc.path}</span>
            </div>
            {doc.doc_url ? (
              <a
                href={doc.doc_url}
                className="mt-2 block truncate text-sm text-sky-400 hover:underline"
                target="_blank"
                rel="noreferrer"
              >
                {doc.doc_url}
              </a>
            ) : null}
            {doc.tags.length > 0 ? (
              <div className="mt-2 text-xs text-zinc-500">tags: {doc.tags.join(", ")}</div>
            ) : null}
            {doc.note ? (
              <div className="mt-3 rounded-lg border border-zinc-800 bg-zinc-950 p-3 text-sm text-zinc-300 whitespace-pre-wrap">
                {doc.note}
              </div>
            ) : null}
            <div className="mt-4 text-sm text-zinc-300 whitespace-pre-wrap font-mono leading-relaxed">{doc.content}</div>
          </article>
        ) : null}
      </div>
    </div>
  );
}
