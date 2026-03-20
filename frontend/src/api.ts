import type {
  BulkIndexIn,
  DocumentIn,
  IndexResponse,
  SearchQuery,
  SearchResponse,
} from "./types";

type ErrorBody = { error?: string };

const API_BASE_URL =
  (import.meta.env.VITE_API_BASE_URL as string | undefined) ?? "http://127.0.0.1:8080";

function getJwtToken(): string | null {
  try {
    return window.localStorage.getItem("inkly.jwt");
  } catch {
    return null;
  }
}

async function apiFetch<T>(path: string, init: RequestInit): Promise<T> {
  const token = getJwtToken();
  const headers = new Headers(init.headers);

  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }

  const url = `${API_BASE_URL}${path}`;
  const res = await fetch(url, { ...init, headers });

  let body: unknown = null;
  const contentType = res.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    body = await res.json().catch(() => null);
  } else {
    body = await res.text().catch(() => null);
  }

  if (!res.ok) {
    const err = (body as ErrorBody | null)?.error;
    throw new Error(err ?? `Request failed: ${res.status}`);
  }

  return body as T;
}

export async function indexDocument(doc: DocumentIn): Promise<IndexResponse> {
  return apiFetch<IndexResponse>("/v1/documents", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(doc),
  });
}

export async function indexDocumentsBulk(bulk: BulkIndexIn): Promise<IndexResponse> {
  return apiFetch<IndexResponse>("/v1/documents/bulk", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(bulk),
  });
}

export async function search(query: SearchQuery): Promise<SearchResponse> {
  const params = new URLSearchParams({
    q: query.q,
    limit: String(query.limit),
  });
  return apiFetch<SearchResponse>(`/v1/search?${params.toString()}`, {
    method: "GET",
  });
}

