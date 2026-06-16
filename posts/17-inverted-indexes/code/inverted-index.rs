// inverted-index.rs
// Core implementation of an Inverted Index for exact keyword matching
// Part of Post #17: Inverted Indexes Explained

use std::collections::{HashMap, HashSet};

/// An Inverted Index that maps Terms to Document IDs
///
/// # Structure
/// - Dictionary: HashMap of terms to postings lists
/// - Postings List: Sorted Vec of document IDs that contain the term
///
/// # Example
/// ```
/// let mut index = InvertedIndex::new();
/// index.add_document_tags(1, vec!["shoes".to_string(), "blue".to_string()]);
/// index.add_document_tags(2, vec!["shoes".to_string(), "red".to_string()]);
///
/// let results = index.search_and(&["shoes", "blue"]);
/// assert_eq!(results, vec![1]);
/// ```
#[derive(Default, Debug, Clone)]
pub struct InvertedIndex {
    /// Map: Term to Sorted List of DocIDs
    index: HashMap<String, Vec<usize>>,

    /// Reverse index: DocID to List of Terms (for deletion)
    reverse: HashMap<usize, Vec<String>>,
}

impl InvertedIndex {
    /// Create a new empty inverted index
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            reverse: HashMap::new(),
        }
    }

    /// Get the number of unique terms in the index
    pub fn num_terms(&self) -> usize {
        self.index.len()
    }

    /// Get the total number of postings (term-document pairs)
    pub fn num_postings(&self) -> usize {
        self.index.values().map(|list| list.len()).sum()
    }

    /// Get the number of documents indexed
    pub fn num_documents(&self) -> usize {
        self.reverse.len()
    }

    /// Add a document with free-form text
    ///
    /// Text is tokenized (lowercase, split on whitespace, remove punctuation)
    /// and stopwords are filtered out.
    ///
    /// # Example
    /// ```
    /// let mut index = InvertedIndex::new();
    /// index.add_document(1, "The quick brown fox jumps over the lazy dog");
    ///
    /// // "The" and "the" are stopwords, removed
    /// let results = index.search("quick");
    /// assert_eq!(results, Some(&vec![1]));
    /// ```
    pub fn add_document(&mut self, doc_id: usize, text: &str) {
        let tokens = self.tokenize(text);

        // Track terms for reverse index
        let mut doc_terms = Vec::new();

        for token in tokens {
            // Add to forward index
            let list = self.index.entry(token.clone()).or_insert_with(Vec::new);

            // Only add if not already present (maintain sorted + unique)
            if list.last() != Some(&doc_id) {
                list.push(doc_id);
            }

            // Track for reverse index
            doc_terms.push(token);
        }

        // Update reverse index
        self.reverse.insert(doc_id, doc_terms);
    }

    /// Add a document with a list of tags/keywords
    ///
    /// This is more efficient than free-text if you already have
    /// structured tags (e.g., product categories, metadata fields).
    ///
    /// # Example
    /// ```
    /// let mut index = InvertedIndex::new();
    /// index.add_document_tags(1, vec![
    ///     "shoes".to_string(),
    ///     "running".to_string(),
    ///     "blue".to_string(),
    /// ]);
    /// ```
    pub fn add_document_tags(&mut self, doc_id: usize, tags: Vec<String>) {
        let mut doc_terms = Vec::new();

        for tag in tags {
            let normalized = tag.to_lowercase().trim().to_string();
            if normalized.is_empty() {
                continue;
            }

            // Add to forward index
            let list = self
                .index
                .entry(normalized.clone())
                .or_insert_with(Vec::new);

            if list.last() != Some(&doc_id) {
                list.push(doc_id);
            }

            // Track for reverse index
            doc_terms.push(normalized);
        }

        // Update reverse index
        self.reverse.insert(doc_id, doc_terms);
    }

    /// Remove a document from the index
    ///
    /// Uses the reverse index to find all terms associated with the
    /// document, then removes the document ID from each postings list.
    ///
    /// # Complexity
    /// O(T x log N) where T = number of terms in document, N = avg postings list size
    pub fn remove_document(&mut self, doc_id: usize) {
        // Get all terms for this document
        if let Some(terms) = self.reverse.remove(&doc_id) {
            for term in terms {
                if let Some(list) = self.index.get_mut(&term) {
                    // Binary search since list is sorted
                    if let Ok(idx) = list.binary_search(&doc_id) {
                        list.remove(idx);
                    }

                    // Remove term entry if list is now empty
                    if list.is_empty() {
                        self.index.remove(&term);
                    }
                }
            }
        }
    }

    /// Tokenize text into searchable terms
    ///
    /// Pipeline:
    /// 1. Lowercase
    /// 2. Split on whitespace
    /// 3. Remove punctuation
    /// 4. Filter empty strings
    /// 5. Remove stopwords
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|s| !s.is_empty())
            .filter(|s| !self.is_stopword(s))
            .map(|s| s.to_string())
            .collect()
    }

    /// Check if a word is a common stopword
    ///
    /// Stopwords are extremely common words that don't provide
    /// meaningful search value (e.g., "the", "a", "and").
    ///
    /// In production, use a more complete list (e.g., 400+ words).
    fn is_stopword(&self, word: &str) -> bool {
        matches!(
            word,
            "the"
                | "a"
                | "an"
                | "and"
                | "or"
                | "but"
                | "in"
                | "on"
                | "at"
                | "to"
                | "for"
                | "of"
                | "with"
                | "by"
                | "from"
                | "as"
                | "is"
                | "was"
                | "are"
                | "were"
                | "be"
                | "been"
                | "being"
        )
    }

    /// Search for a single term
    ///
    /// Returns the postings list (sorted DocIDs) or None if term not found.
    ///
    /// # Example
    /// ```
    /// let results = index.search("shoes");
    /// if let Some(doc_ids) = results {
    ///     println!("Found in documents: {:?}", doc_ids);
    /// }
    /// ```
    pub fn search(&self, term: &str) -> Option<&Vec<usize>> {
        let normalized = term.to_lowercase();
        self.index.get(&normalized)
    }

    /// Search with AND logic (all terms must match)
    ///
    /// Returns documents that contain ALL of the specified terms.
    /// Uses efficient two-pointer intersection algorithm.
    ///
    /// # Algorithm
    /// 1. Get postings lists for all terms
    /// 2. Sort by list length (shortest first)
    /// 3. Intersect iteratively, starting with shortest
    /// 4. Early termination if result becomes empty
    ///
    /// # Complexity
    /// O(T x N) where T = number of terms, N = avg postings list size
    ///
    /// # Example
    /// ```
    /// // Find blue Nike shoes
    /// let results = index.search_and(&["shoes", "blue", "nike"]);
    /// ```
    pub fn search_and(&self, terms: &[&str]) -> Vec<usize> {
        if terms.is_empty() {
            return Vec::new();
        }

        // Get postings lists for all terms
        let mut lists: Vec<&Vec<usize>> =
            terms.iter().filter_map(|term| self.search(term)).collect();

        // If any term is not found, no results
        if lists.len() != terms.len() {
            return Vec::new();
        }

        // Sort by list length (intersect shortest first for efficiency)
        lists.sort_by_key(|list| list.len());

        // Start with shortest list
        let mut result = lists[0].clone();

        // Intersect with remaining lists
        for list in &lists[1..] {
            result = intersect(&result, list);

            // Early termination if result becomes empty
            if result.is_empty() {
                return result;
            }
        }

        result
    }

    /// Search with OR logic (any term matches)
    ///
    /// Returns documents that contain ANY of the specified terms.
    /// Uses HashSet for deduplication and fast insertion.
    ///
    /// # Complexity
    /// O(T x N) where T = number of terms, N = avg postings list size
    ///
    /// # Example
    /// ```
    /// // Find Nike OR Adidas products
    /// let results = index.search_or(&["nike", "adidas"]);
    /// ```
    pub fn search_or(&self, terms: &[&str]) -> Vec<usize> {
        let mut result_set = HashSet::new();

        for term in terms {
            if let Some(list) = self.search(term) {
                for &doc_id in list {
                    result_set.insert(doc_id);
                }
            }
        }

        let mut result: Vec<_> = result_set.into_iter().collect();
        result.sort_unstable();
        result
    }

    /// Search with NOT logic (exclude term)
    ///
    /// Returns documents that contain `include_term` but NOT `exclude_term`.
    ///
    /// # Example
    /// ```
    /// // Find shoes that are NOT red
    /// let results = index.search_not("shoes", "red");
    /// ```
    pub fn search_not(&self, include_term: &str, exclude_term: &str) -> Vec<usize> {
        let include_list = match self.search(include_term) {
            Some(list) => list,
            None => return Vec::new(),
        };

        let exclude_list = match self.search(exclude_term) {
            Some(list) => list,
            None => return include_list.clone(),
        };

        difference(include_list, exclude_list)
    }

    /// Optimize the index by sorting and deduplicating all postings lists
    ///
    /// Call this if documents were added out of order, or after bulk inserts.
    /// Not needed if documents are always added with increasing IDs.
    pub fn optimize(&mut self) {
        for list in self.index.values_mut() {
            list.sort_unstable();
            list.dedup();
        }
    }

    /// Get all terms in the index (sorted)
    pub fn get_terms(&self) -> Vec<String> {
        let mut terms: Vec<_> = self.index.keys().cloned().collect();
        terms.sort();
        terms
    }

    /// Get statistics about the index
    pub fn stats(&self) -> IndexStats {
        let mut list_sizes: Vec<usize> = self.index.values().map(|list| list.len()).collect();
        list_sizes.sort_unstable();

        let total_postings: usize = list_sizes.iter().sum();
        let avg_postings = if !list_sizes.is_empty() {
            total_postings as f64 / list_sizes.len() as f64
        } else {
            0.0
        };

        let median_postings = if !list_sizes.is_empty() {
            list_sizes[list_sizes.len() / 2]
        } else {
            0
        };

        IndexStats {
            num_terms: self.num_terms(),
            num_documents: self.num_documents(),
            total_postings,
            avg_postings_per_term: avg_postings,
            median_postings_per_term: median_postings,
            max_postings_per_term: list_sizes.last().copied().unwrap_or(0),
            min_postings_per_term: list_sizes.first().copied().unwrap_or(0),
        }
    }
}

/// Statistics about an inverted index
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub num_terms: usize,
    pub num_documents: usize,
    pub total_postings: usize,
    pub avg_postings_per_term: f64,
    pub median_postings_per_term: usize,
    pub max_postings_per_term: usize,
    pub min_postings_per_term: usize,
}

impl std::fmt::Display for IndexStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InvertedIndex Statistics:\n\
             - Terms: {}\n\
             - Documents: {}\n\
             - Total Postings: {}\n\
             - Avg Postings/Term: {:.2}\n\
             - Median Postings/Term: {}\n\
             - Max Postings/Term: {}\n\
             - Min Postings/Term: {}",
            self.num_terms,
            self.num_documents,
            self.total_postings,
            self.avg_postings_per_term,
            self.median_postings_per_term,
            self.max_postings_per_term,
            self.min_postings_per_term
        )
    }
}

/// Intersect two sorted lists in O(n + m) time using two-pointer algorithm
///
/// # Algorithm
/// ```
/// i = 0, j = 0
/// while i < len(a) && j < len(b):
///     if a[i] < b[j]:
///         i++
///     elif a[i] > b[j]:
///         j++
///     else:
///         result.push(a[i])
///         i++, j++
/// ```
///
/// # Example
/// ```
/// let a = vec![1, 3, 5, 7, 9];
/// let b = vec![2, 3, 5, 8, 10];
/// let result = intersect(&a, &b);
/// assert_eq!(result, vec![3, 5]);
/// ```
pub fn intersect(list_a: &[usize], list_b: &[usize]) -> Vec<usize> {
    let mut i = 0;
    let mut j = 0;
    let mut result = Vec::new();

    while i < list_a.len() && j < list_b.len() {
        if list_a[i] < list_b[j] {
            i += 1; // Advance left pointer
        } else if list_a[i] > list_b[j] {
            j += 1; // Advance right pointer
        } else {
            // Match found
            result.push(list_a[i]);
            i += 1;
            j += 1;
        }
    }
    result
}

/// Compute difference A - B (elements in A but not in B)
///
/// # Example
/// ```
/// let a = vec![1, 2, 3, 4, 5];
/// let b = vec![2, 4];
/// let result = difference(&a, &b);
/// assert_eq!(result, vec![1, 3, 5]);
/// ```
pub fn difference(list_a: &[usize], list_b: &[usize]) -> Vec<usize> {
    let mut i = 0;
    let mut j = 0;
    let mut result = Vec::new();

    while i < list_a.len() {
        if j >= list_b.len() {
            // Rest of A is result
            result.extend_from_slice(&list_a[i..]);
            break;
        }

        if list_a[i] < list_b[j] {
            // In A but not in B
            result.push(list_a[i]);
            i += 1;
        } else if list_a[i] > list_b[j] {
            j += 1;
        } else {
            // In both, skip
            i += 1;
            j += 1;
        }
    }

    result
}

// =============================================================================
// Example Usage & Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_indexing() {
        let mut index = InvertedIndex::new();

        // Add documents
        index.add_document_tags(
            1,
            vec![
                "shoes".to_string(),
                "running".to_string(),
                "blue".to_string(),
            ],
        );
        index.add_document_tags(
            2,
            vec![
                "shoes".to_string(),
                "running".to_string(),
                "red".to_string(),
            ],
        );
        index.add_document_tags(
            3,
            vec![
                "shoes".to_string(),
                "casual".to_string(),
                "white".to_string(),
            ],
        );
        index.add_document_tags(
            4,
            vec![
                "hat".to_string(),
                "baseball".to_string(),
                "blue".to_string(),
            ],
        );

        assert_eq!(index.num_documents(), 4);
        assert_eq!(index.num_terms(), 8);
    }

    #[test]
    fn test_single_term_search() {
        let mut index = InvertedIndex::new();
        index.add_document_tags(1, vec!["shoes".to_string()]);
        index.add_document_tags(2, vec!["shoes".to_string()]);
        index.add_document_tags(3, vec!["hat".to_string()]);

        let results = index.search("shoes");
        assert_eq!(results, Some(&vec![1, 2]));

        let results = index.search("hat");
        assert_eq!(results, Some(&vec![3]));

        let results = index.search("nonexistent");
        assert_eq!(results, None);
    }

    #[test]
    fn test_and_query() {
        let mut index = InvertedIndex::new();
        index.add_document_tags(
            1,
            vec!["shoes".to_string(), "blue".to_string(), "nike".to_string()],
        );
        index.add_document_tags(
            2,
            vec!["shoes".to_string(), "red".to_string(), "adidas".to_string()],
        );
        index.add_document_tags(
            3,
            vec!["hat".to_string(), "blue".to_string(), "nike".to_string()],
        );

        // shoes AND blue
        let results = index.search_and(&["shoes", "blue"]);
        assert_eq!(results, vec![1]);

        // blue AND nike
        let results = index.search_and(&["blue", "nike"]);
        assert_eq!(results, vec![1, 3]);

        // shoes AND blue AND nike
        let results = index.search_and(&["shoes", "blue", "nike"]);
        assert_eq!(results, vec![1]);
    }

    #[test]
    fn test_or_query() {
        let mut index = InvertedIndex::new();
        index.add_document_tags(1, vec!["nike".to_string()]);
        index.add_document_tags(2, vec!["adidas".to_string()]);
        index.add_document_tags(3, vec!["nike".to_string()]);

        let results = index.search_or(&["nike", "adidas"]);
        assert_eq!(results, vec![1, 2, 3]);
    }

    #[test]
    fn test_not_query() {
        let mut index = InvertedIndex::new();
        index.add_document_tags(1, vec!["shoes".to_string(), "blue".to_string()]);
        index.add_document_tags(2, vec!["shoes".to_string(), "red".to_string()]);
        index.add_document_tags(3, vec!["shoes".to_string(), "white".to_string()]);

        // shoes BUT NOT red
        let results = index.search_not("shoes", "red");
        assert_eq!(results, vec![1, 3]);
    }

    #[test]
    fn test_document_removal() {
        let mut index = InvertedIndex::new();
        index.add_document_tags(1, vec!["shoes".to_string(), "blue".to_string()]);
        index.add_document_tags(2, vec!["shoes".to_string(), "red".to_string()]);

        assert_eq!(index.num_documents(), 2);

        index.remove_document(1);

        assert_eq!(index.num_documents(), 1);
        assert_eq!(index.search("shoes"), Some(&vec![2]));
        assert_eq!(index.search("blue"), None); // No more docs with blue
    }

    #[test]
    fn test_text_tokenization() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "The quick brown fox jumps over the lazy dog");

        // "the" is a stopword, should be filtered
        assert_eq!(index.search("the"), None);

        // Other words should be indexed
        assert_eq!(index.search("quick"), Some(&vec![1]));
        assert_eq!(index.search("brown"), Some(&vec![1]));
        assert_eq!(index.search("fox"), Some(&vec![1]));
    }

    #[test]
    fn test_intersection_algorithm() {
        let a = vec![1, 3, 5, 7, 9];
        let b = vec![2, 3, 5, 8, 10];
        let result = intersect(&a, &b);
        assert_eq!(result, vec![3, 5]);

        // Edge cases
        let empty: Vec<usize> = Vec::new();
        assert_eq!(intersect(&a, &empty), Vec::<usize>::new());
        assert_eq!(intersect(&empty, &b), Vec::<usize>::new());

        // No overlap
        let c = vec![1, 2, 3];
        let d = vec![4, 5, 6];
        assert_eq!(intersect(&c, &d), Vec::<usize>::new());

        // Full overlap
        let e = vec![1, 2, 3];
        let f = vec![1, 2, 3];
        assert_eq!(intersect(&e, &f), vec![1, 2, 3]);
    }

    #[test]
    fn test_difference_algorithm() {
        let a = vec![1, 2, 3, 4, 5];
        let b = vec![2, 4];
        let result = difference(&a, &b);
        assert_eq!(result, vec![1, 3, 5]);

        // Edge cases
        let empty: Vec<usize> = Vec::new();
        assert_eq!(difference(&a, &empty), a);
        assert_eq!(difference(&empty, &b), Vec::<usize>::new());
    }
}

// =============================================================================
// Example: E-Commerce Product Search
// =============================================================================

fn main() {
    println!("=== Inverted Index Demo: E-Commerce Search ===\n");

    let mut index = InvertedIndex::new();

    // Build product catalog
    println!("Building product catalog...");
    let products = vec![
        (1, vec!["shoes", "running", "blue", "nike"]),
        (2, vec!["shoes", "running", "red", "adidas"]),
        (3, vec!["shoes", "casual", "white", "nike"]),
        (4, vec!["hat", "baseball", "blue", "nike"]),
        (5, vec!["shoes", "hiking", "brown", "merrell"]),
    ];

    for (doc_id, tags) in products {
        let tags_owned: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
        index.add_document_tags(doc_id, tags_owned);
        println!("  Product {}: {:?}", doc_id, tags);
    }

    println!("\n{}\n", index.stats());

    // Query 1: Single term
    println!("Query 1: Find all shoes");
    if let Some(results) = index.search("shoes") {
        println!("  Results: {:?}\n", results);
    }

    // Query 2: AND query
    println!("Query 2: Find blue Nike products");
    let results = index.search_and(&["blue", "nike"]);
    println!("  Results: {:?}\n", results);

    // Query 3: Complex AND
    println!("Query 3: Find blue running shoes");
    let results = index.search_and(&["shoes", "running", "blue"]);
    println!("  Results: {:?}\n", results);

    // Query 4: OR query
    println!("Query 4: Find Nike OR Adidas products");
    let results = index.search_or(&["nike", "adidas"]);
    println!("  Results: {:?}\n", results);

    // Query 5: NOT query
    println!("Query 5: Find shoes BUT NOT red");
    let results = index.search_not("shoes", "red");
    println!("  Results: {:?}\n", results);

    // Remove a product
    println!("Removing product 2 (red adidas shoes)...");
    index.remove_document(2);

    println!("\nAfter removal:");
    if let Some(results) = index.search("shoes") {
        println!("  Shoes: {:?}", results);
    }
    if let Some(results) = index.search("red") {
        println!("  Red: {:?}", results);
    } else {
        println!("  Red: None (term removed)");
    }
}
