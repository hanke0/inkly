use std::path::Path;

use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, Query, QueryParser, TermQuery};
use tantivy::schema::{Field, IndexRecordOption, STORED, STRING, TEXT, Value};
use tantivy::{doc, schema, Index, Term};

use crate::error::{Result, SearchError};

#[derive(Clone, Debug)]
pub struct IndexStats {
    pub indexed: u64,
    pub deleted: u64,
}

#[derive(Clone, Debug)]
pub struct SearchResultItem {
    pub doc_id: String,
    pub title: String,
    pub snippet: String,
    pub score: f32,
}

#[derive(Clone)]
pub struct IndexManager {
    index: Index,
    tenant_id_field: Field,
    doc_key_field: Field,
    doc_id_field: Field,
    title_field: Field,
    content_field: Field,
}

impl IndexManager {
    pub fn open_or_create<P: AsRef<Path>>(index_dir: P) -> Result<Self> {
        let index_dir = index_dir.as_ref();
        std::fs::create_dir_all(index_dir)?;

        let index = if let Ok(existing) = Index::open_in_dir(index_dir) {
            existing
        } else {
            let schema = Self::build_schema();
            let index = Index::create_in_dir(index_dir, schema)?;
            index
        };

        let schema = index.schema();
        let tenant_id_field = schema
            .get_field("tenant_id")
            .map_err(|_| SearchError::InvalidInput("missing tenant_id field".into()))?;
        let doc_key_field = schema
            .get_field("doc_key")
            .map_err(|_| SearchError::InvalidInput("missing doc_key field".into()))?;
        let doc_id_field = schema
            .get_field("doc_id")
            .map_err(|_| SearchError::InvalidInput("missing doc_id field".into()))?;
        let title_field = schema
            .get_field("title")
            .map_err(|_| SearchError::InvalidInput("missing title field".into()))?;
        let content_field = schema
            .get_field("content")
            .map_err(|_| SearchError::InvalidInput("missing content field".into()))?;

        Ok(Self {
            index,
            tenant_id_field,
            doc_key_field,
            doc_id_field,
            title_field,
            content_field,
        })
    }

    fn build_schema() -> schema::Schema {
        let mut builder = schema::Schema::builder();

        let _tenant_id = builder.add_text_field("tenant_id", STRING | STORED);
        let _doc_key = builder.add_text_field("doc_key", STRING | STORED);
        let _doc_id = builder.add_text_field("doc_id", STRING | STORED);
        let _title = builder.add_text_field("title", TEXT | STORED);
        let _content = builder.add_text_field("content", TEXT | STORED);

        builder.build()
    }

    fn doc_key(tenant_id: &str, doc_id: &str) -> String {
        format!("{tenant_id}:{doc_id}")
    }

    pub fn index_document(
        &self,
        tenant_id: &str,
        doc_id: &str,
        title: &str,
        content: &str,
    ) -> Result<IndexStats> {
        if tenant_id.trim().is_empty() {
            return Err(SearchError::InvalidInput("tenant_id is empty".into()));
        }
        if doc_id.trim().is_empty() {
            return Err(SearchError::InvalidInput("doc_id is empty".into()));
        }

        let mut writer = self.index.writer(50_000_000)?;

        let key = Self::doc_key(tenant_id, doc_id);
        writer.delete_term(Term::from_field_text(self.doc_key_field, &key));

        let document = doc!(
            self.tenant_id_field => tenant_id,
            self.doc_key_field => key,
            self.doc_id_field => doc_id,
            self.title_field => title,
            self.content_field => content
        );
        writer.add_document(document)?;

        writer.commit()?;

        Ok(IndexStats {
            indexed: 1,
            deleted: 0, // Tantivy doesn't tell us how many were deleted.
        })
    }

    pub fn index_documents(
        &self,
        tenant_id: &str,
        docs: impl IntoIterator<Item = (String, String, String)>,
    ) -> Result<IndexStats> {
        let mut writer = self.index.writer(50_000_000)?;
        let mut indexed = 0u64;

        for (doc_id, title, content) in docs {
            let key = Self::doc_key(tenant_id, &doc_id);
            writer.delete_term(Term::from_field_text(self.doc_key_field, &key));

            let document = doc!(
                self.tenant_id_field => tenant_id,
                self.doc_key_field => key,
                self.doc_id_field => doc_id,
                self.title_field => title,
                self.content_field => content
            );
            writer.add_document(document)?;

            indexed += 1;
        }

        writer.commit()?;

        Ok(IndexStats {
            indexed,
            deleted: 0,
        })
    }

    pub fn search(
        &self,
        tenant_id: &str,
        query_str: &str,
        limit: u32,
    ) -> Result<(u64, Vec<SearchResultItem>)> {
        let query_str = query_str.trim();
        if query_str.is_empty() {
            return Err(SearchError::InvalidInput("q is empty".into()));
        }

        let limit = limit.clamp(1, 50) as usize;
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let tenant_term = Term::from_field_text(self.tenant_id_field, tenant_id);
        let tenant_query = TermQuery::new(tenant_term, IndexRecordOption::Basic);

        let parser = QueryParser::for_index(&self.index, vec![self.title_field, self.content_field]);
        let full_query = parser.parse_query(query_str)?;

        let query = BooleanQuery::new(vec![(Occur::Must, Box::new(tenant_query)), (Occur::Must, full_query)]);

        let top_docs = TopDocs::with_limit(limit);
        let total_hits = query.count(&searcher)? as u64;
        let hits = searcher.search(&query, &top_docs)?;

        let mut results = Vec::with_capacity(hits.len());
        for (score, doc_address) in hits {
            let retrieved = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;
            let title = retrieved
                .get_first(self.title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let content = retrieved
                .get_first(self.content_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let doc_id = retrieved
                .get_first(self.doc_id_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let snippet = content.chars().take(220).collect::<String>();
            results.push(SearchResultItem {
                doc_id,
                title,
                snippet,
                score,
            });
        }

        Ok((total_hits, results))
    }
}

