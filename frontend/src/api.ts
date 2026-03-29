import type {
  BulkIndexIn,
  CatalogResponse,
  DocumentDetailResponse,
  DocumentIn,
  IndexResponse,
  SearchQuery,
  SearchResponse,
  SessionResponse,
} from "./types";

type ErrorBody = { error?: string };

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

  const res = await fetch(path, { ...init, headers });

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

/** Multipart upload: field `file` (UTF-8 text or HTML) plus text fields matching the index form. */
export async function indexDocumentUpload(formData: FormData): Promise<IndexResponse> {
  const headers = new Headers();
  applyBasicAuth(headers);

  const res = await fetch("/v1/documents/upload", { method: "POST", headers, body: formData });

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

export async function fetchCatalog(path: string): Promise<CatalogResponse> {
  const params = new URLSearchParams({ path: path || "/" });
  return apiFetch<CatalogResponse>(`/v1/catalog?${params.toString()}`, {
    method: "GET",
  });
}

export async function fetchDocument(docId: number): Promise<DocumentDetailResponse> {
  return apiFetch<DocumentDetailResponse>(`/v1/documents/${docId}`, {
    method: "GET",
  });
}

export async function fetchSession(): Promise<SessionResponse> {
  return apiFetch<SessionResponse>("/v1/session", { method: "GET" });
}

export function hasStoredCredentials(): boolean {
  try {
    const u = window.localStorage.getItem(LS_USERNAME_KEY);
    const p = window.localStorage.getItem(LS_PASSWORD_KEY);
    return Boolean(u?.trim()) && p !== null;
  } catch {
    return false;
  }
}

export function storeCredentials(username: string, password: string): void {
  window.localStorage.setItem(LS_USERNAME_KEY, username.trim());
  window.localStorage.setItem(LS_PASSWORD_KEY, password);
}

export function clearStoredCredentials(): void {
  window.localStorage.removeItem(LS_USERNAME_KEY);
  window.localStorage.removeItem(LS_PASSWORD_KEY);
}

/** Returns true when the server accepts these Basic credentials (does not persist them). */
export async function verifyLogin(username: string, password: string): Promise<boolean> {
  const headers = new Headers();
  headers.set("Authorization", `Basic ${utf8ToBase64(`${username.trim()}:${password}`)}`);
  const res = await fetch("/v1/session", { method: "GET", headers });
  return res.ok;
}

export { LS_PASSWORD_KEY, LS_USERNAME_KEY };
