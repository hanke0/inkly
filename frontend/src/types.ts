export type DocumentIn = {
  doc_id: number;
  title: string;
  content: string;
  doc_url: string;
  created_at: number;
  updated_at: number;
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

