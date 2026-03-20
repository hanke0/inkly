import { useEffect, useMemo, useState } from "react";

import { indexDocument, search } from "./api";
import type { DocumentIn, IndexResponse, SearchQuery, SearchResponse } from "./types";

const LS_KEY = "inkly.jwt";

type Mode = "search" | "index";

export default function App() {
  const apiBaseUrl = useMemo(
    () => (import.meta.env.VITE_API_BASE_URL as string | undefined) ?? "http://127.0.0.1:8080",
    [],
  );

  const [mode, setMode] = useState<Mode>("search");

  const [jwt, setJwt] = useState<string>("");
  const [tokenStatus, setTokenStatus] = useState<string>("");

  const [docId, setDocId] = useState<number>(1);
  const [title, setTitle] = useState<string>("Hello");
  const [content, setContent] = useState<string>("This is a test document.");
  const [docUrl, setDocUrl] = useState<string>("https://example.com/doc/1");
  const [createdTimestamp, setCreatedTimestamp] = useState<number>(() => Math.floor(Date.now() / 1000));
  const [updateTimestamp, setUpdateTimestamp] = useState<number>(() => Math.floor(Date.now() / 1000));
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
    const stored = window.localStorage.getItem(LS_KEY);
    if (stored) {
      setJwt(stored);
      setTokenStatus("Token loaded from localStorage.");
    }
  }, []);

  function saveToken() {
    setError("");
    setIndexRes(null);
    setSearchRes(null);

    if (!jwt.trim()) {
      setTokenStatus("Please paste a JWT token first.");
      return;
    }
    window.localStorage.setItem(LS_KEY, jwt.trim());
    setTokenStatus("Token saved. Requests will use it.");
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

    const payload: DocumentIn = {
      doc_id: docId,
      title: title.trim(),
      content,
      doc_url: docUrl.trim(),
      created_at: createdTimestamp,
      updated_at: updateTimestamp,
      tags,
      path: path.trim(),
      note,
    };

    try {
      const res = await indexDocument(payload);
      setIndexRes(res);
      setTokenStatus("Indexed successfully.");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Index request failed.");
      setTokenStatus("");
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
      setTokenStatus("Search complete.");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Search request failed.");
      setTokenStatus("");
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

          <div className="mt-4">
            <label className="block text-sm text-zinc-300">JWT (Authorization: Bearer ...)</label>
            <textarea
              className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 font-mono text-sm outline-none focus:border-zinc-700"
              rows={3}
              value={jwt}
              onChange={(e) => setJwt(e.target.value)}
              placeholder="Paste your JWT token here..."
            />
            <div className="mt-2 flex items-center justify-between gap-3">
              <button
                type="button"
                className="rounded-lg bg-zinc-200 px-3 py-1 text-sm font-medium text-zinc-900 disabled:opacity-50"
                onClick={saveToken}
              >
                Save token
              </button>
              <div className="text-xs text-zinc-400">{tokenStatus}</div>
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
                <label className="block text-sm text-zinc-300">content</label>
                <textarea
                  className="mt-2 h-28 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 font-mono text-sm outline-none focus:border-zinc-700"
                  value={content}
                  onChange={(e) => setContent(e.target.value)}
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

              <div className="mt-3 grid gap-3 md:grid-cols-2">
                <div>
                  <label className="block text-sm text-zinc-300">created_at (unix seconds)</label>
                  <input
                    type="number"
                    className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 text-sm outline-none focus:border-zinc-700"
                    value={createdTimestamp}
                    onChange={(e) => setCreatedTimestamp(Number(e.target.value))}
                  />
                </div>
                <div>
                  <label className="block text-sm text-zinc-300">updated_at (unix seconds)</label>
                  <input
                    type="number"
                    className="mt-2 w-full rounded-lg border border-zinc-800 bg-zinc-950 p-2 text-sm outline-none focus:border-zinc-700"
                    value={updateTimestamp}
                    onChange={(e) => setUpdateTimestamp(Number(e.target.value))}
                  />
                </div>
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
                <div className="text-xs text-zinc-400">Tenant scoping is enforced server-side via JWT.</div>
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

