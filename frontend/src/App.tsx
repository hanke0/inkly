import { useEffect, useMemo, useRef, useState } from "react";

import { indexDocument, indexDocumentUpload, LS_PASSWORD_KEY, LS_USERNAME_KEY, search } from "./api";
import type { DocumentIn, IndexResponse, SearchQuery, SearchResponse } from "./types";

type Mode = "search" | "index";

export default function App() {
  const apiBaseUrl = useMemo(
    () => (import.meta.env.VITE_API_BASE_URL as string | undefined) ?? "http://127.0.0.1:8080",
    [],
  );

  const [mode, setMode] = useState<Mode>("search");

  const [authUsername, setAuthUsername] = useState<string>("");
  const [authPassword, setAuthPassword] = useState<string>("");
  const [authStatus, setAuthStatus] = useState<string>("");

  const [docId, setDocId] = useState<number>(1);
  const [title, setTitle] = useState<string>("Hello");
  const [content, setContent] = useState<string>("This is a test document.");
  const [contentFile, setContentFile] = useState<File | null>(null);
  const contentFileInputRef = useRef<HTMLInputElement>(null);
  const [docUrl, setDocUrl] = useState<string>("https://example.com/doc/1");
  const [tagsText, setTagsText] = useState<string>("test,example");
  const [path, setPath] = useState<string>("/");
  const [note, setNote] = useState<string>("Optional note...");

  const [q, setQ] = useState<string>("test");
  const [limit, setLimit] = useState<number>(10);

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>("");

  const [indexRes, setIndexRes] = useState<IndexResponse | null>(null);
  const [searchRes, setSearchRes] = useState<SearchResponse | null>(null);

  useEffect(() => {
    const u = window.localStorage.getItem(LS_USERNAME_KEY);
    const p = window.localStorage.getItem(LS_PASSWORD_KEY);
    if (u) {
      setAuthUsername(u);
    }
    if (p !== null) {
      setAuthPassword(p);
    }
    if (u?.trim() && p !== null) {
      setAuthStatus("Credentials loaded from localStorage.");
    }
  }, []);

  function saveCredentials() {
    setError("");
    setIndexRes(null);
    setSearchRes(null);

    if (!authUsername.trim()) {
      setAuthStatus("Please enter a username.");
      return;
    }
    window.localStorage.setItem(LS_USERNAME_KEY, authUsername.trim());
    window.localStorage.setItem(LS_PASSWORD_KEY, authPassword);
    setAuthStatus("Credentials saved. Requests will use HTTP Basic auth.");
  }

  async function onIndexSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setIndexRes(null);
    setSearchRes(null);
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
        fd.append("doc_id", String(docId));
        fd.append("title", title.trim());
        fd.append("doc_url", docUrl.trim());
        fd.append("path", path.trim());
        fd.append("note", note);
        fd.append("tags", tagsText);
        res = await indexDocumentUpload(fd);
      } else {
        if (!content.trim()) {
          setError("Add content in the text area or choose a UTF-8 text file.");
          setAuthStatus("");
          setLoading(false);
          return;
        }
        const payload: DocumentIn = {
          doc_id: docId,
          title: title.trim(),
          content,
          doc_url: docUrl.trim(),
          tags,
          path: path.trim(),
          note,
        };
        res = await indexDocument(payload);
      }
      setIndexRes(res);
      setAuthStatus("Indexed successfully.");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Index request failed.");
      setAuthStatus("");
    } finally {
      setLoading(false);
    }
  }

  async function onSearchSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setIndexRes(null);
    setSearchRes(null);
    setLoading(true);

    const query: SearchQuery = {
      q: q.trim(),
      limit: Math.max(1, Math.min(50, limit)),
    };

    try {
      const res = await search(query);
      setSearchRes(res);
      setAuthStatus("Search complete.");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Search request failed.");
      setAuthStatus("");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100">
      <div className="mx-auto max-w-3xl px-4 py-10">
        <div className="mb-6">
          <div className="text-sm text-zinc-400">Inkly SPA</div>
          <div className="text-2xl font-semibold">Axum + Tantivy</div>
          <div className="mt-2 text-sm text-zinc-400">API base: {apiBaseUrl}</div>
        </div>

        <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
          <div className="flex gap-3">
            <button
              className={`rounded-lg px-3 py-1 text-sm ${mode === "search" ? "bg-zinc-200 text-zinc-900" : "bg-zinc-800 text-zinc-200"}`}
              onClick={() => setMode("search")}
              type="button"
            >
              Search
            </button>
            <button
              className={`rounded-lg px-3 py-1 text-sm ${mode === "index" ? "bg-zinc-200 text-zinc-900" : "bg-zinc-800 text-zinc-200"}`}
              onClick={() => setMode("index")}
              type="button"
            >
              Index
            </button>
          </div>

          <div className="mt-4 space-y-3">
            <div className="text-sm text-zinc-300">HTTP Basic auth (must match server USERNAME / PASSWORD)</div>
            <div>
              <label className="block text-sm text-zinc-400">Username</label>
              <input
                type="text"
                autoComplete="username"
                className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 font-mono text-sm outline-none focus:border-zinc-700"
                value={authUsername}
                onChange={(e) => setAuthUsername(e.target.value)}
                placeholder="inkly"
              />
            </div>
            <div>
              <label className="block text-sm text-zinc-400">Password</label>
              <input
                type="password"
                autoComplete="current-password"
                className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 font-mono text-sm outline-none focus:border-zinc-700"
                value={authPassword}
                onChange={(e) => setAuthPassword(e.target.value)}
                placeholder="••••••••"
              />
            </div>
            <div className="flex items-center justify-between gap-3">
              <button
                type="button"
                className="rounded-lg bg-zinc-200 px-3 py-1 text-sm font-medium text-zinc-900 disabled:opacity-50"
                onClick={saveCredentials}
              >
                Save credentials
              </button>
              <div className="text-xs text-zinc-400">{authStatus}</div>
            </div>
          </div>
        </div>

        {error ? (
          <div className="mt-4 rounded-xl border border-red-900 bg-red-950/30 p-3 text-sm text-red-200">
            {error}
          </div>
        ) : null}

        {mode === "index" ? (
          <div className="mt-6 rounded-xl border border-zinc-800 bg-zinc-900 p-4">
            <form onSubmit={onIndexSubmit}>
              <div className="grid gap-3 md:grid-cols-2">
                <div>
                  <label className="block text-sm text-zinc-300">doc_id</label>
                  <input
                    type="number"
                    className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 text-sm outline-none focus:border-zinc-700"
                    value={docId}
                    onChange={(e) => setDocId(Number(e.target.value))}
                  />
                </div>
                <div>
                  <label className="block text-sm text-zinc-300">title</label>
                  <input
                    className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 text-sm outline-none focus:border-zinc-700"
                    value={title}
                    onChange={(e) => setTitle(e.target.value)}
                  />
                </div>
              </div>

              <div className="mt-3">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <label className="block text-sm text-zinc-300">content</label>
                  <div className="flex flex-wrap items-center gap-2">
                    <input
                      ref={contentFileInputRef}
                      type="file"
                      accept=".txt,.md,.markdown,text/plain,text/markdown"
                      className="max-w-full text-xs text-zinc-400 file:mr-2 file:rounded-md file:border-0 file:bg-zinc-800 file:px-2 file:py-1 file:text-zinc-200"
                      onChange={(e) => {
                        const f = e.target.files?.[0];
                        setContentFile(f ?? null);
                      }}
                    />
                    {contentFile ? (
                      <button
                        type="button"
                        className="rounded-md bg-zinc-800 px-2 py-1 text-xs text-zinc-200"
                        onClick={() => {
                          setContentFile(null);
                          if (contentFileInputRef.current) {
                            contentFileInputRef.current.value = "";
                          }
                        }}
                      >
                        Clear file
                      </button>
                    ) : null}
                  </div>
                </div>
                {contentFile ? (
                  <div className="mt-2 text-xs text-zinc-400">
                    Indexing will use the file ({contentFile.name}). The textarea is ignored until you clear the file.
                  </div>
                ) : null}
                <textarea
                  className="mt-2 h-28 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 font-mono text-sm outline-none focus:border-zinc-700 disabled:opacity-50"
                  value={content}
                  onChange={(e) => setContent(e.target.value)}
                  disabled={Boolean(contentFile)}
                />
              </div>

              <div className="mt-3 grid gap-3 md:grid-cols-2">
                <div>
                  <label className="block text-sm text-zinc-300">doc_url</label>
                  <input
                    className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 text-sm outline-none focus:border-zinc-700"
                    value={docUrl}
                    onChange={(e) => setDocUrl(e.target.value)}
                  />
                </div>
                <div>
                  <label className="block text-sm text-zinc-300">path</label>
                  <input
                    className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 text-sm outline-none focus:border-zinc-700"
                    value={path}
                    onChange={(e) => setPath(e.target.value)}
                  />
                </div>
              </div>

              <div className="mt-3 text-xs text-zinc-400">
                `created_at` / `updated_at` are set automatically by the API.
              </div>

              <div className="mt-3">
                <label className="block text-sm text-zinc-300">tags (comma-separated)</label>
                <input
                  className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 text-sm outline-none focus:border-zinc-700"
                  value={tagsText}
                  onChange={(e) => setTagsText(e.target.value)}
                />
              </div>

              <div className="mt-3">
                <label className="block text-sm text-zinc-300">note</label>
                <textarea
                  className="mt-2 h-20 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 font-mono text-sm outline-none focus:border-zinc-700"
                  value={note}
                  onChange={(e) => setNote(e.target.value)}
                />
              </div>

              <div className="mt-3 flex items-center justify-between gap-3">
                <button
                  type="submit"
                  disabled={loading}
                  className="rounded-lg bg-zinc-200 px-3 py-2 text-sm font-medium text-zinc-900 disabled:opacity-50"
                >
                  {loading ? "Indexing..." : "Index document"}
                </button>
                <div className="text-xs text-zinc-400">Will replace existing doc with same `doc_id` within tenant.</div>
              </div>
            </form>

            {indexRes ? (
              <div className="mt-4 rounded-lg border border-zinc-800 bg-zinc-950 p-3 text-sm">
                <div className="text-zinc-200 font-medium">Index response</div>
                <div className="mt-2 font-mono text-xs text-zinc-300">
                  {JSON.stringify(indexRes, null, 2)}
                </div>
              </div>
            ) : null}
          </div>
        ) : (
          <div className="mt-6 rounded-xl border border-zinc-800 bg-zinc-900 p-4">
            <form onSubmit={onSearchSubmit}>
              <div className="grid gap-3 md:grid-cols-2">
                <div>
                  <label className="block text-sm text-zinc-300">q</label>
                  <input
                    className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 text-sm outline-none focus:border-zinc-700"
                    value={q}
                    onChange={(e) => setQ(e.target.value)}
                  />
                </div>
                <div>
                  <label className="block text-sm text-zinc-300">limit</label>
                  <input
                    type="number"
                    className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 text-sm outline-none focus:border-zinc-700"
                    value={limit}
                    onChange={(e) => setLimit(Number(e.target.value))}
                    min={1}
                    max={50}
                  />
                </div>
              </div>

              <div className="mt-3 flex items-center justify-between gap-3">
                <button
                  type="submit"
                  disabled={loading}
                  className="rounded-lg bg-zinc-200 px-3 py-2 text-sm font-medium text-zinc-900 disabled:opacity-50"
                >
                  {loading ? "Searching..." : "Search"}
                </button>
                <div className="text-xs text-zinc-400">Tenant scoping uses the Basic auth username.</div>
              </div>
            </form>

            {searchRes ? (
              <div className="mt-4">
                <div className="flex items-center justify-between">
                  <div className="text-zinc-200 text-sm font-medium">Results</div>
                  <div className="text-xs text-zinc-400">total_hits: {searchRes.total_hits}</div>
                </div>
                <div className="mt-3 space-y-3">
                  {searchRes.results.length === 0 ? (
                    <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-3 text-sm text-zinc-400">
                      No matches.
                    </div>
                  ) : null}
                  {searchRes.results.map((r) => (
                    <div
                      key={r.doc_id}
                      className="rounded-lg border border-zinc-800 bg-zinc-950 p-3"
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0">
                          <div className="truncate text-sm font-medium text-zinc-100">{r.title}</div>
                          <div className="mt-1 text-xs text-zinc-400 font-mono">{r.doc_id}</div>
                          <div className="mt-1 text-xs text-zinc-500 font-mono truncate">
                            {r.doc_url}
                          </div>
                        </div>
                        <div className="text-xs text-zinc-400 font-mono">{r.score.toFixed(3)}</div>
                      </div>
                      <div className="mt-2 text-sm text-zinc-300 font-mono">{r.snippet}</div>
                      <div className="mt-2 text-xs text-zinc-400">
                        <div className="font-mono truncate">path: {r.path}</div>
                        <div className="font-mono truncate">tags: {r.tags.join(", ")}</div>
                        <div className="font-mono mt-1 text-zinc-500 truncate">{r.note}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ) : null}
          </div>
        )}
      </div>
    </div>
  );
}

