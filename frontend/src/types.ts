export type DocumentIn = {
  /** Omit for server-assigned id. */
  doc_id?: number;
  title: string;
  content: string;
  doc_url: string;
  tags: string[];
  path: string;
  note: string;
};

export type BulkIndexIn = {
  documents: DocumentIn[];
};

export type IndexResponse = {
  indexed: number;
  deleted: number;
  /** Single-document index: assigned id when `doc_id` was omitted. */
  doc_id?: number;
  /** Bulk index: final ids in request order. */
  doc_ids?: number[];
};

export type SearchQuery = {
  q: string;
  limit: number;
};

export type SearchResult = {
  doc_id: number;
  title: string;
  doc_url: string;
  snippet: string;
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
  doc_url: string;
  path: string;
  note: string;
  tags: string[];
  created_at: number;
  updated_at: number;
};

