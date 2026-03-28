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

const LS_USERNAME_KEY = "inkly.basic.username";
const LS_PASSWORD_KEY = "inkly.basic.password";

function utf8ToBase64(s: string): string {
  const bytes = new TextEncoder().encode(s);
  let binary = "";
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary);
}

function getBasicAuthHeader(): string | null {
  try {
    const user = window.localStorage.getItem(LS_USERNAME_KEY);
    const pass = window.localStorage.getItem(LS_PASSWORD_KEY);
    if (!user?.trim() || pass === null) {
      return null;
    }
    return `Basic ${utf8ToBase64(`${user.trim()}:${pass}`)}`;
  } catch {
    return null;
  }
}

function applyBasicAuth(headers: Headers): void {
  const auth = getBasicAuthHeader();
  if (auth) {
    headers.set("Authorization", auth);
  }
}

async function apiFetch<T>(path: string, init: RequestInit): Promise<T> {
  const headers = new Headers(init.headers);
  applyBasicAuth(headers);

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

/** Multipart upload: field `file` (UTF-8 text) plus text fields matching the index form. */
export async function indexDocumentUpload(formData: FormData): Promise<IndexResponse> {
  const headers = new Headers();
  applyBasicAuth(headers);

  const url = `${API_BASE_URL}/v1/documents/upload`;
  const res = await fetch(url, { method: "POST", headers, body: formData });

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

  return body as IndexResponse;
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

export { LS_PASSWORD_KEY, LS_USERNAME_KEY };
