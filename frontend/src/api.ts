import type {
  CatalogResponse,
  DocumentDetailResponse,
  IndexResponse,
  SearchQuery,
  SearchResponse,
  SessionResponse,
  SummaryEnqueueResponse,
} from './types';
import { announceApiError } from './lib/apiErrorNotify';
import { extractErrorMessage } from './lib/errors';

type ErrorBody = { error?: string };

/** When set, sent as `Accept-Language` on API calls (browser default is overridden). */
let preferredAcceptLanguage: string | null = null;

export function setPreferredAcceptLanguage(value: string | null): void {
  preferredAcceptLanguage = value;
}

const LS_USERNAME_KEY = 'inkly.basic.username';
const LS_PASSWORD_KEY = 'inkly.basic.password';

function utf8ToBase64(s: string): string {
  const bytes = new TextEncoder().encode(s);
  let binary = '';
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
    headers.set('Authorization', auth);
  }
}

type ApiFetchOptions = {
  /** When true, do not open the global error dialog (still throws). */
  quiet?: boolean;
};

type CompressionEncoding = 'zstd' | 'br' | 'gzip' | 'deflate';

const REQUEST_COMPRESSION_PREFERENCE: readonly CompressionEncoding[] = [
  'zstd',
  'br',
  'gzip',
  'deflate',
];

function createCompressionStream(encoding: CompressionEncoding): CompressionStream {
  return new CompressionStream(encoding as unknown as CompressionFormat);
}

function pickSupportedRequestCompression(): CompressionEncoding | null {
  if (typeof CompressionStream === 'undefined') {
    return null;
  }
  for (const encoding of REQUEST_COMPRESSION_PREFERENCE) {
    try {
      // Probe runtime support for this algorithm.
      createCompressionStream(encoding);
      return encoding;
    } catch {
      // Try the next preferred algorithm.
    }
  }
  return null;
}

async function compressFormDataBody(formData: FormData): Promise<{
  body: ArrayBuffer;
  contentType: string;
  contentEncoding: CompressionEncoding;
} | null> {
  const contentEncoding = pickSupportedRequestCompression();
  if (contentEncoding === null) {
    return null;
  }

  const encoded = new Response(formData);
  const contentType = encoded.headers.get('content-type');
  if (!contentType) {
    return null;
  }

  const compression = createCompressionStream(contentEncoding);
  const writer = compression.writable.getWriter();
  const source = new Uint8Array(await encoded.arrayBuffer());
  await writer.write(source);
  await writer.close();

  const compressed = await new Response(compression.readable).arrayBuffer();
  return { body: compressed, contentType, contentEncoding };
}

async function apiFetch<T>(
  path: string,
  init: RequestInit,
  options?: ApiFetchOptions,
): Promise<T> {
  const headers = new Headers(init.headers);
  applyBasicAuth(headers);
  if (preferredAcceptLanguage) {
    headers.set('Accept-Language', preferredAcceptLanguage);
  }

  let res: Response;
  try {
    res = await fetch(path, { ...init, headers });
  } catch (e) {
    if (!options?.quiet) {
      const text = extractErrorMessage(e, '').trim();
      announceApiError(
        text !== ''
          ? { source: 'text', text }
          : { source: 'i18n', key: 'errors.fetchFailed' },
      );
    }
    throw e;
  }

  if (res.status === 204) {
    return undefined as T;
  }

  let body: unknown = null;
  const contentType = res.headers.get('content-type') ?? '';
  if (contentType.includes('application/json')) {
    body = await res.json().catch(() => null);
  } else {
    body = await res.text().catch(() => null);
  }

  if (!res.ok) {
    const err = (body as ErrorBody | null)?.error;
    const message = err ?? `Request failed: ${res.status}`;
    if (!options?.quiet) {
      announceApiError({ source: 'text', text: message });
    }
    throw new Error(message);
  }

  return body as T;
}

/** Multipart create: field `file` (UTF-8 text or HTML); server always assigns `doc_id`. */
export async function indexDocumentUpload(
  formData: FormData,
): Promise<IndexResponse> {
  const compressed = await compressFormDataBody(formData);
  return apiFetch<IndexResponse>('/v1/documents', {
    method: 'POST',
    headers: compressed
      ? {
          'Content-Type': compressed.contentType,
          'Content-Encoding': compressed.contentEncoding,
        }
      : undefined,
    body: compressed ? compressed.body : formData,
  });
}

/** Update document fields via multipart form-data. */
export async function updateDocument(
  docId: number,
  formData: FormData,
): Promise<IndexResponse> {
  const compressed = await compressFormDataBody(formData);
  return apiFetch<IndexResponse>(`/v1/documents/${docId}`, {
    method: 'POST',
    headers: compressed
      ? {
          'Content-Type': compressed.contentType,
          'Content-Encoding': compressed.contentEncoding,
        }
      : undefined,
    body: compressed ? compressed.body : formData,
  });
}

/** Queue async summary generation; `enqueued` is false when the job was already pending. */
export async function enqueueDocumentSummary(
  docId: number,
): Promise<SummaryEnqueueResponse> {
  return apiFetch<SummaryEnqueueResponse>(`/v1/documents/${docId}/summary`, {
    method: 'POST',
  });
}

export async function search(query: SearchQuery): Promise<SearchResponse> {
  const params = new URLSearchParams({
    q: query.q,
    limit: String(query.limit),
  });
  if (query.path != null && query.path !== '' && query.path !== '/') {
    params.set('path', query.path);
  }
  if (query.tags != null && query.tags.trim() !== '') {
    params.set('tags', query.tags.trim());
  }
  return apiFetch<SearchResponse>(`/v1/search?${params.toString()}`, {
    method: 'GET',
  });
}

export async function fetchCatalog(path: string): Promise<CatalogResponse> {
  const params = new URLSearchParams({ path: path || '/' });
  return apiFetch<CatalogResponse>(`/v1/catalog?${params.toString()}`, {
    method: 'GET',
  });
}

export async function fetchDocument(
  docId: number,
): Promise<DocumentDetailResponse> {
  const headers = new Headers();
  applyBasicAuth(headers);
  if (preferredAcceptLanguage) {
    headers.set('Accept-Language', preferredAcceptLanguage);
  }

  const res = await fetch(`/v1/documents/${docId}`, { method: 'GET', headers });
  if (!res.ok) {
    let message = `Request failed: ${res.status}`;
    const contentType = res.headers.get('content-type') ?? '';
    if (contentType.includes('application/json')) {
      const body = (await res.json().catch(() => null)) as ErrorBody | null;
      if (body?.error) {
        message = body.error;
      }
    } else {
      const text = (await res.text().catch(() => '')).trim();
      if (text !== '') {
        message = text;
      }
    }
    announceApiError({ source: 'text', text: message });
    throw new Error(message);
  }

  const contentType = res.headers.get('content-type') ?? '';
  if (!contentType.includes('multipart/form-data')) {
    return (await res.json()) as DocumentDetailResponse;
  }

  const form = await res.formData();
  const filePart = form.get('file');
  const content =
    filePart instanceof File ? await filePart.text() : String(filePart ?? '');

  const tagsRaw = String(form.get('tags') ?? '').trim();
  return {
    doc_id: Number(form.get('doc_id') ?? docId),
    title: String(form.get('title') ?? ''),
    content,
    summary: String(form.get('summary') ?? ''),
    doc_url: String(form.get('doc_url') ?? ''),
    path: String(form.get('path') ?? '/'),
    note: String(form.get('note') ?? ''),
    tags:
      tagsRaw === ''
        ? []
        : tagsRaw
            .split(',')
            .map((t) => t.trim())
            .filter((t) => t !== ''),
    created_at: Number(form.get('created_at') ?? 0),
    updated_at: Number(form.get('updated_at') ?? 0),
  };
}

export async function deleteDocument(docId: number): Promise<void> {
  return apiFetch<void>(`/v1/documents/${docId}`, { method: 'DELETE' });
}

export async function fetchSession(options?: {
  quiet?: boolean;
}): Promise<SessionResponse> {
  return apiFetch<SessionResponse>(
    '/v1/session',
    { method: 'GET' },
    options?.quiet ? { quiet: true } : undefined,
  );
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
export async function verifyLogin(
  username: string,
  password: string,
): Promise<boolean> {
  const headers = new Headers();
  headers.set(
    'Authorization',
    `Basic ${utf8ToBase64(`${username.trim()}:${password}`)}`,
  );
  if (preferredAcceptLanguage) {
    headers.set('Accept-Language', preferredAcceptLanguage);
  }
  try {
    const res = await fetch('/v1/session', { method: 'GET', headers });
    return res.ok;
  } catch (e) {
    const text = extractErrorMessage(e, '').trim();
    announceApiError(
      text !== ''
        ? { source: 'text', text }
        : { source: 'i18n', key: 'errors.fetchFailed' },
    );
    throw e;
  }
}

export { LS_PASSWORD_KEY, LS_USERNAME_KEY };
