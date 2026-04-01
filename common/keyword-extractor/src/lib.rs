use std::collections::{HashMap, HashSet};

const STOPWORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
    "from", "as", "is", "was", "are", "were", "be", "been", "being", "have", "has", "had",
    "do", "does", "did", "will", "would", "could", "should", "may", "might", "shall", "can",
    "not", "no", "nor", "so", "if", "then", "than", "that", "this", "these", "those", "it",
    "its", "he", "she", "we", "they", "me", "him", "her", "us", "them", "my", "his", "our",
    "your", "their", "what", "which", "who", "whom", "how", "when", "where", "why", "all",
    "each", "every", "both", "few", "more", "most", "other", "some", "such", "only", "own",
    "same", "too", "very", "just", "about", "above", "after", "before", "between", "into",
    "through", "during", "out", "up", "down", "over", "under", "again", "further",
];

fn stopword_set() -> HashSet<&'static str> {
    STOPWORDS.iter().copied().collect()
}

pub fn tokenize(text: &str) -> Vec<String> {
    let stops = stopword_set();
    text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() >= 2 && !stops.contains(w.as_str()))
        .collect()
}

pub fn term_frequencies(tokens: &[String]) -> Vec<(String, f64)> {
    let total = tokens.len() as f64;
    if total == 0.0 {
        return Vec::new();
    }
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for t in tokens {
        *counts.entry(t.as_str()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(|(term, count)| (term.to_string(), count as f64 / total))
        .collect()
}

pub fn extract_keywords(
    doc_tf: &[(String, f64)],
    corpus_doc_count: usize,
    term_doc_counts: &HashMap<String, usize>,
    max_keywords: usize,
) -> Vec<String> {
    let n = (corpus_doc_count.max(1)) as f64;
    let mut scored: Vec<(String, f64)> = doc_tf
        .iter()
        .map(|(term, tf)| {
            let df = term_doc_counts.get(term).copied().unwrap_or(1).max(1) as f64;
            let idf = (n / df).ln() + 1.0;
            (term.clone(), tf * idf)
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(max_keywords).map(|(t, _)| t).collect()
}

pub fn serialize_keywords(keywords: &[String]) -> String {
    keywords.join("\n")
}

pub fn deserialize_keywords(data: &str) -> Vec<String> {
    data.lines()
        .map(|l| l.to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_filters_stopwords_and_short() {
        let tokens = tokenize("The quick brown fox is a very fast animal");
        assert!(tokens.contains(&"quick".to_string()));
        assert!(tokens.contains(&"brown".to_string()));
        assert!(tokens.contains(&"fox".to_string()));
        assert!(tokens.contains(&"fast".to_string()));
        assert!(tokens.contains(&"animal".to_string()));
        assert!(!tokens.contains(&"the".to_string()));
        assert!(!tokens.contains(&"is".to_string()));
        assert!(!tokens.contains(&"a".to_string()));
    }

    #[test]
    fn tokenize_lowercases() {
        let tokens = tokenize("Hello WORLD");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn tokenize_splits_punctuation() {
        let tokens = tokenize("hello,world.test");
        assert_eq!(tokens, vec!["hello", "world", "test"]);
    }

    #[test]
    fn term_frequencies_basic() {
        let tokens = vec!["hello".into(), "world".into(), "hello".into()];
        let tf = term_frequencies(&tokens);
        let map: HashMap<String, f64> = tf.into_iter().collect();
        assert!((map["hello"] - 2.0 / 3.0).abs() < 0.001);
        assert!((map["world"] - 1.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn term_frequencies_empty() {
        let tf = term_frequencies(&[]);
        assert!(tf.is_empty());
    }

    #[test]
    fn extract_keywords_ranks_by_tfidf() {
        let doc_tf = vec![
            ("rare".to_string(), 0.5),
            ("common".to_string(), 0.5),
        ];
        let mut term_doc_counts = HashMap::new();
        term_doc_counts.insert("rare".to_string(), 1);
        term_doc_counts.insert("common".to_string(), 10);

        let kw = extract_keywords(&doc_tf, 10, &term_doc_counts, 2);
        assert_eq!(kw[0], "rare");
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let kw = vec!["hello".to_string(), "world".to_string()];
        let serialized = serialize_keywords(&kw);
        let deserialized = deserialize_keywords(&serialized);
        assert_eq!(kw, deserialized);
    }

    #[test]
    fn deserialize_ignores_empty_lines() {
        let data = "hello\n\nworld\n";
        let kw = deserialize_keywords(data);
        assert_eq!(kw, vec!["hello", "world"]);
    }
}
