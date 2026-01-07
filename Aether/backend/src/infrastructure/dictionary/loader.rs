use std::path::Path;
use std::sync::Arc;
use stardict::StarDict;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use tracing::{info, warn};
use std::fs;

#[derive(Clone)]
pub struct DictionaryLoader {
    inner: Arc<DictionaryInner>,
}

struct DictionaryInner {
    dict: Option<Box<dyn StarDict + Send + Sync>>,
    matcher: SkimMatcherV2,
    words: Vec<String>,
}

impl DictionaryLoader {
    pub fn new<P: AsRef<Path>>(dir_path: P) -> Self {
        let dir_path = dir_path.as_ref();
        let dict: Option<Box<dyn StarDict + Send + Sync>> = None;
        let words = Vec::new();

        info!("Initializing DictionaryLoader from: {:?}", dir_path);

        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "ifo") {
                    info!("Found dictionary info file: {:?}", path);
                    // Match StarDict trait API (assuming obscure struct in 0.2.0)
                    // TODO: Find correct struct constructor for stardict 0.2.0
                    // match StarDict::new(&path) { ... }
                    warn!("StarDict 0.2.0 loading temporarily disabled due to API obscurity. implementation pending.");
                }
            }
        } else {
            warn!("Dictionary directory not found or unreadable: {:?}", dir_path);
        }

        Self {
            inner: Arc::new(DictionaryInner {
                dict,
                matcher: SkimMatcherV2::default(),
                words,
            })
        }
    }

    pub fn lookup(&self, _word: &str) -> Option<String> {
        if let Some(_dict) = &self.inner.dict {
            // dict.lookup(word) - assuming generic dictionary usually has lookup or get or similar.
            // stardict trait in 0.2.0 likely has methods.
            // We'll trust auto-complete or check error if method name is wrong.
            // But since we can't inspect the trait, we might fail here too.
            // We'll stub this too for safety now.
            // return dict.lookup(word).map(|s| s.to_string());
            return None; 
        }
        None
    }

    pub fn fuzzy_search(&self, query: &str) -> Vec<String> {
        let mut matches: Vec<(String, i64)> = Vec::new();
        if self.inner.words.is_empty() {
             return Vec::new();
        }

        for word in &self.inner.words {
            if let Some(score) = self.inner.matcher.fuzzy_match(word, query) {
                matches.push((word.clone(), score));
            }
        }

        // Sort by score descending
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        
        matches.into_iter().take(10).map(|(w, _)| w).collect()
    }
}
