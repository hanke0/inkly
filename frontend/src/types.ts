export type DocumentIn = {
  doc_id: string;
  title: string;
  content: string;
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
  doc_id: string;
  title: string;
  snippet: string;
  score: number;
};

export type SearchResponse = {
  total_hits: number;
  results: SearchResult[];
};

