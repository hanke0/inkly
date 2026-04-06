export type DocumentIn = {
  /** Not used for `POST /v1/documents` multipart (server assigns id). Optional in JSON for other flows. */
  doc_id?: number;
  title: string;
  /** Required for new documents. Omit on updates (existing content is preserved). */
  content?: string;
  doc_url: string;
  tags: string[];
  path: string;
  note: string;
};

export type IndexResponse = {
  indexed: number;
  deleted: number;
  /** Single-document index: assigned id when `doc_id` was omitted. */
  doc_id?: number;
  /** Reserved; empty in current API responses. */
  doc_ids?: number[];
};

/** `POST /v1/documents/{doc_id}/summary` */
export type SummaryEnqueueResponse = {
  enqueued: boolean;
  message: string;
};

export type SearchQuery = {
  q: string;
  limit: number;
  /** Normalized folder path; subtree filter (this folder and below). */
  path?: string;
  /** Comma-separated tags (document must have every tag). */
  tags?: string;
};

export type SearchResult = {
  doc_id: number;
  title: string;
  doc_url: string;
  snippet: string;
  summary: string;
  score: number;
  created_at: number;
  updated_at: number;
  tags: string[];
  path: string;
  note: string;
};

export type SearchResponse = {
  total_hits: number;
  results: SearchResult[];
};

export type SessionResponse = {
  ok: boolean;
  /** Resolved server-side from `Accept-Language`. */
  locale: string;
};

export type CatalogSubdir = {
  name: string;
  path: string;
};

export type CatalogFile = {
  doc_id: number;
  title: string;
};

export type CatalogResponse = {
  path: string;
  subdirs: CatalogSubdir[];
  files: CatalogFile[];
};

export type DocumentDetailResponse = {
  doc_id: number;
  title: string;
  content: string;
  summary: string;
  doc_url: string;
  path: string;
  note: string;
  tags: string[];
  created_at: number;
  updated_at: number;
};
