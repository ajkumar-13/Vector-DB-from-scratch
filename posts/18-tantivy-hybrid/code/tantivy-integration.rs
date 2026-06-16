// tantivy-integration.rs
// Wrapper around Tantivy for metadata indexing and filtering
// Part of Post #18: The Hybrid Engine

use serde_json::{json, Value as JsonValue};
use std::path::Path;
use std::sync::Mutex;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{TantivyDocument, Index, IndexReader, IndexWriter, ReloadPolicy, Term};

/// Metadata Index using Tantivy for full-text search and filtering
///
/// # Architecture
/// - Stores JSON metadata for each document
/// - Uses `point_id` field to align with HNSW PointIds
/// - Provides fast boolean queries (AND/OR/NOT)
/// - Generates bitmasks for O(1) filtering during HNSW search
///
/// # Example
/// ```
/// let mut index = MetadataIndex::new()?;
///
/// // Index documents
/// index.index_metadata(0, json!({"category": "shoes", "price": 89.99}))?;
/// index.index_metadata(1, json!({"category": "hats", "price": 24.99}))?;
/// index.commit()?;
///
/// // Search and generate bitmask
/// let bitmask = index.search_to_bitmask("category:shoes AND price:<100", 1000)?;
/// assert!(bitmask[0]);  // Doc 0 matches
/// assert!(!bitmask[1]); // Doc 1 does not match
/// ```
pub struct MetadataIndex {
    index: Index,
    schema: Schema,

    // Field handles (stored once, reused many times)
    point_id_field: Field,
    metadata_field: Field,

    // Writer and reader
    writer: Mutex<IndexWriter>,
    reader: IndexReader,
}

impl MetadataIndex {
    /// Create a new in-memory Tantivy index
    pub fn new() -> Result<Self, tantivy::TantivyError> {
        Self::with_path(None)
    }

    /// Create a new Tantivy index (in-memory or on disk)
    ///
    /// # Arguments
    /// * `path` - Optional directory path. If None, creates in-memory index.
    ///
    /// # Schema
    /// - `point_id` (u64): INDEXED | FAST | STORED - Links to HNSW PointId
    /// - `metadata` (JSON): TEXT | STORED - User's arbitrary JSON data
    pub fn with_path(path: Option<&str>) -> Result<Self, tantivy::TantivyError> {
        // Build schema
        let mut schema_builder = Schema::builder();

        // The ID connecting us to HNSW
        // INDEXED: Can search by point_id (e.g., point_id:42)
        // FAST: Columnar storage for fast sequential access
        // STORED: Returned in search results
        let point_id_field = schema_builder.add_u64_field("point_id", INDEXED | FAST | STORED);

        // Generic JSON blob for user metadata
        // TEXT: Full-text search with tokenization
        // STORED: Return original JSON in results
        let metadata_field = schema_builder.add_json_field("metadata", TEXT | STORED);

        let schema = schema_builder.build();

        // Create index (in RAM or on disk)
        let index = if let Some(dir_path) = path {
            std::fs::create_dir_all(dir_path)?;
            Index::create_in_dir(dir_path, schema.clone())?
        } else {
            Index::create_in_ram(schema.clone())
        };

        // Create writer with 50 MB buffer
        // This buffer accumulates changes before flushing to segments
        let writer = index.writer(50_000_000)?;

        // Create reader with auto-reload policy
        // OnCommit: Reader automatically sees new data after commits
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(Self {
            index,
            schema,
            point_id_field,
            metadata_field,
            writer: Mutex::new(writer),
            reader,
        })
    }

    /// Index a document's metadata
    ///
    /// # Arguments
    /// * `point_id` - The HNSW PointId (0, 1, 2, ...)
    /// * `payload` - JSON metadata (user-defined structure)
    ///
    /// # Example
    /// ```
    /// index.index_metadata(42, json!({
    ///     "title": "Running Shoes",
    ///     "category": "footwear",
    ///     "price": 89.99,
    ///     "tags": ["running", "sports", "blue"]
    /// }))?;
    /// ```
    pub fn index_metadata(
        &self,
        point_id: u64,
        payload: JsonValue,
    ) -> Result<(), tantivy::TantivyError> {
        let mut doc = TantivyDocument::default();

        // Critical: Store the HNSW ID explicitly
        // This is our bridge between the two indexes
        doc.add_u64(self.point_id_field, point_id);

        // Store the JSON payload
        // Tantivy will automatically index all fields for full-text search
        let owned = tantivy::schema::OwnedValue::from(payload);
        if let tantivy::schema::OwnedValue::Object(map) = owned {
            doc.add_object(self.metadata_field, map);
        }

        // Add to index (buffered in memory)
        let mut writer = self.writer.lock().unwrap();
        writer.add_document(doc)?;

        Ok(())
    }

    /// Commit all pending writes
    ///
    /// This flushes the in-memory buffer to disk (or RAM index) and makes
    /// the changes visible to readers.
    ///
    /// # Performance
    /// - Commits are relatively expensive (~10-50ms)
    /// - Batch multiple inserts before committing
    /// - For real-time updates, commit after each batch of writes
    pub fn commit(&self) -> Result<(), tantivy::TantivyError> {
        let mut writer = self.writer.lock().unwrap();
        writer.commit()?;
        Ok(())
    }

    /// Delete a document by point_id
    ///
    /// # Note
    /// Tantivy deletions are logical (tombstones). Space is reclaimed
    /// during segment merging.
    pub fn delete_by_point_id(&self, point_id: u64) -> Result<(), tantivy::TantivyError> {
        let mut writer = self.writer.lock().unwrap();

        // Create a term query for point_id
        let term = Term::from_field_u64(self.point_id_field, point_id);

        // Delete all documents matching this term
        writer.delete_term(term);

        Ok(())
    }

    /// Execute a query and return a bitmask of matching point IDs
    ///
    /// # Arguments
    /// * `query_str` - Query string (Tantivy query syntax)
    /// * `max_id` - Maximum point_id (size of bitmask to allocate)
    ///
    /// # Query Syntax Examples
    /// - `category:shoes` - Single term
    /// - `category:shoes AND price:<100` - Boolean AND with range
    /// - `brand:Nike OR brand:Adidas` - Boolean OR
    /// - `tags:running AND NOT tags:women` - Boolean NOT
    /// - `"running shoes"` - Phrase query
    ///
    /// # Returns
    /// Vec<bool> where bitmask[point_id] = true if document matches
    ///
    /// # Performance
    /// - Query parsing: approximately 10-50 us
    /// - Inverted index search: approximately 100-500 us (1M docs)
    /// - Bitmask construction: approximately 50-200 us (for 10K matches)
    /// - Total: approximately 1ms typical
    pub fn search_to_bitmask(
        &self,
        query_str: &str,
        max_id: usize,
    ) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
        let searcher = self.reader.searcher();

        // Parse the query
        // QueryParser handles AND/OR/NOT logic, field targeting, ranges, etc.
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.metadata_field], // Default search field
        );
        let query = query_parser.parse_query(query_str)?;

        // Collect ALL matching documents
        // TopDocs(usize::MAX) means "return everything"
        let top_docs = searcher.search(&query, &TopDocs::with_limit(usize::MAX))?;

        // Initialize bitmask (all false)
        let mut bitmask = vec![false; max_id];

        // Iterate over all matching documents
        for (_score, doc_address) in top_docs {
            // Get the segment reader for this document
            let segment_reader = searcher.segment_reader(doc_address.segment_ord);

            // Get fast field reader for this segment
            // Fast fields are columnar storage (all point_ids in sequence)
            let point_id_reader = segment_reader.fast_fields().u64("point_id")?;

            // Read point_id (O(1) access in columnar format)
            let point_id = point_id_reader.first(doc_address.doc_id).unwrap_or(0) as usize;

            // Set bitmask
            if point_id < max_id {
                bitmask[point_id] = true;
            }
        }

        Ok(bitmask)
    }

    /// Execute a query and return matching point IDs as a sorted Vec
    ///
    /// Use this if you need a list of IDs rather than a bitmask.
    pub fn search_to_ids(&self, query_str: &str) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
        let searcher = self.reader.searcher();
        let query_parser = QueryParser::for_index(&self.index, vec![self.metadata_field]);
        let query = query_parser.parse_query(query_str)?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(usize::MAX))?;
        let mut ids = Vec::new();

        for (_score, doc_address) in top_docs {
            let segment_reader = searcher.segment_reader(doc_address.segment_ord);
            let point_id_reader = segment_reader.fast_fields().u64("point_id")?;
            let point_id = point_id_reader.first(doc_address.doc_id).unwrap_or(0) as usize;
            ids.push(point_id);
        }

        ids.sort_unstable();
        Ok(ids)
    }

    /// Get the number of documents in the index
    pub fn num_docs(&self) -> usize {
        let searcher = self.reader.searcher();
        searcher.num_docs() as usize
    }

    /// Estimate filter selectivity (percentage of docs that match)
    ///
    /// Returns a value between 0.0 and 1.0
    ///
    /// # Use Case
    /// Query planning: decide whether to use pre-filtering or post-filtering
    pub fn estimate_selectivity(&self, query_str: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let matching_docs = self.search_to_ids(query_str)?.len();
        let total_docs = self.num_docs();

        if total_docs == 0 {
            return Ok(0.0);
        }

        Ok(matching_docs as f64 / total_docs as f64)
    }

    /// Retrieve a document's metadata by point_id
    pub fn get_metadata(
        &self,
        point_id: u64,
    ) -> Result<Option<JsonValue>, Box<dyn std::error::Error>> {
        let searcher = self.reader.searcher();

        // Search for this specific point_id
        let term = Term::from_field_u64(self.point_id_field, point_id);
        let query = Box::new(tantivy::query::TermQuery::new(
            term,
            tantivy::schema::IndexRecordOption::Basic,
        ));

        let top_docs = searcher.search(&*query, &TopDocs::with_limit(1))?;

        if top_docs.is_empty() {
            return Ok(None);
        }

        let (_score, doc_address) = top_docs[0];
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

        // Extract metadata field
        if let Some(field_value) = retrieved_doc.get_first(self.metadata_field) {
            if let Some(obj) = field_value.as_object() {
                let json_str = serde_json::to_string(&obj.into_iter().collect::<Vec<_>>()).unwrap_or_default();
                if let Ok(val) = serde_json::from_str(&json_str) {
                    return Ok(Some(val));
                }
            }
        }

        Ok(None)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_indexing() {
        let index = MetadataIndex::new().unwrap();

        index
            .index_metadata(0, json!({"category": "shoes"}))
            .unwrap();
        index
            .index_metadata(1, json!({"category": "hats"}))
            .unwrap();
        index.commit().unwrap();

        assert_eq!(index.num_docs(), 2);
    }

    #[test]
    fn test_search_to_bitmask() {
        let index = MetadataIndex::new().unwrap();

        index
            .index_metadata(0, json!({"category": "shoes", "price": 89}))
            .unwrap();
        index
            .index_metadata(1, json!({"category": "hats", "price": 24}))
            .unwrap();
        index
            .index_metadata(2, json!({"category": "shoes", "price": 120}))
            .unwrap();
        index.commit().unwrap();

        let bitmask = index.search_to_bitmask("category:shoes", 10).unwrap();
        assert!(bitmask[0]);
        assert!(!bitmask[1]);
        assert!(bitmask[2]);
    }

    #[test]
    fn test_boolean_and_query() {
        let index = MetadataIndex::new().unwrap();

        index
            .index_metadata(0, json!({"category": "shoes", "brand": "Nike"}))
            .unwrap();
        index
            .index_metadata(1, json!({"category": "shoes", "brand": "Adidas"}))
            .unwrap();
        index
            .index_metadata(2, json!({"category": "hats", "brand": "Nike"}))
            .unwrap();
        index.commit().unwrap();

        let bitmask = index
            .search_to_bitmask("category:shoes AND brand:Nike", 10)
            .unwrap();
        assert!(bitmask[0]);
        assert!(!bitmask[1]);
        assert!(!bitmask[2]);
    }

    #[test]
    fn test_range_query() {
        let index = MetadataIndex::new().unwrap();

        index.index_metadata(0, json!({"price": 50})).unwrap();
        index.index_metadata(1, json!({"price": 100})).unwrap();
        index.index_metadata(2, json!({"price": 150})).unwrap();
        index.commit().unwrap();

        let bitmask = index.search_to_bitmask("price:<100", 10).unwrap();
        assert!(bitmask[0]);
        assert!(!bitmask[1]);
        assert!(!bitmask[2]);
    }

    #[test]
    fn test_deletion() {
        let index = MetadataIndex::new().unwrap();

        index
            .index_metadata(0, json!({"category": "shoes"}))
            .unwrap();
        index
            .index_metadata(1, json!({"category": "hats"}))
            .unwrap();
        index.commit().unwrap();

        assert_eq!(index.num_docs(), 2);

        index.delete_by_point_id(0).unwrap();
        index.commit().unwrap();

        let bitmask = index.search_to_bitmask("category:shoes", 10).unwrap();
        assert!(!bitmask[0]); // Deleted
    }

    #[test]
    fn test_get_metadata() {
        let index = MetadataIndex::new().unwrap();

        let original = json!({"title": "Test", "value": 42});
        index.index_metadata(5, original.clone()).unwrap();
        index.commit().unwrap();

        let retrieved = index.get_metadata(5).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap()["value"], 42);
    }

    #[test]
    fn test_selectivity_estimation() {
        let index = MetadataIndex::new().unwrap();

        for i in 0..100 {
            let category = if i < 10 { "rare" } else { "common" };
            index
                .index_metadata(i, json!({"category": category}))
                .unwrap();
        }
        index.commit().unwrap();

        let selectivity = index.estimate_selectivity("category:rare").unwrap();
        assert!((selectivity - 0.1).abs() < 0.01); // approximately 10%
    }
}

// =============================================================================
// Example: E-Commerce Search
// =============================================================================

fn main() {
    println!("=== Tantivy Integration Demo ===\n");

    let index = MetadataIndex::new().unwrap();

    // Index products
    println!("Indexing products...");
    let products = vec![
        (
            0,
            json!({"title": "Nike Air Max", "category": "shoes", "price": 120, "tags": ["running", "blue"]}),
        ),
        (
            1,
            json!({"title": "Adidas Ultra Boost", "category": "shoes", "price": 180, "tags": ["running", "white"]}),
        ),
        (
            2,
            json!({"title": "Nike Air Jordan", "category": "shoes", "price": 150, "tags": ["basketball", "red"]}),
        ),
        (
            3,
            json!({"title": "Reebok Classic", "category": "shoes", "price": 85, "tags": ["casual", "white"]}),
        ),
        (
            4,
            json!({"title": "Baseball Cap", "category": "hats", "price": 25, "tags": ["baseball", "blue"]}),
        ),
    ];

    for (point_id, metadata) in products {
        index.index_metadata(point_id, metadata).unwrap();
    }

    index.commit().unwrap();
    println!("Indexed {} documents\n", index.num_docs());

    // Query 1: Category filter
    println!("Query 1: category:shoes");
    let bitmask = index.search_to_bitmask("category:shoes", 10).unwrap();
    print_bitmask("Matches", &bitmask);

    // Query 2: Price range
    println!("\nQuery 2: price:<100");
    let bitmask = index.search_to_bitmask("price:<100", 10).unwrap();
    print_bitmask("Matches", &bitmask);

    // Query 3: Boolean AND
    println!("\nQuery 3: category:shoes AND price:<100");
    let bitmask = index
        .search_to_bitmask("category:shoes AND price:<100", 10)
        .unwrap();
    print_bitmask("Matches", &bitmask);

    // Query 4: Tag search
    println!("\nQuery 4: tags:running");
    let bitmask = index.search_to_bitmask("tags:running", 10).unwrap();
    print_bitmask("Matches", &bitmask);

    // Selectivity analysis
    println!("\n=== Selectivity Analysis ===");
    let queries = vec![
        "category:shoes",
        "price:<100",
        "category:shoes AND price:<100",
        "tags:blue",
    ];

    for query in queries {
        let selectivity = index.estimate_selectivity(query).unwrap();
        println!("{:40} => {:.1}%", query, selectivity * 100.0);
    }
}

fn print_bitmask(label: &str, bitmask: &[bool]) {
    let matches: Vec<usize> = bitmask
        .iter()
        .enumerate()
        .filter(|(_, &matched)| matched)
        .map(|(i, _)| i)
        .collect();
    println!("  {}: {:?}", label, matches);
}
