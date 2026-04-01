use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use strsim::levenshtein;

/// Diagnostics for the sentence anchoring repair process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchoringDiagnostics {
    pub exact_matches: u32,
    pub normalized_matches: u32,
    pub fuzzy_matches: u32,
    pub unresolved: u32,
    pub total_sentences: u32,
}

/// Resolution method used to anchor a sentence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResolutionMethod {
    /// Exact hash match — most reliable
    Exact,
    /// Normalized hash match (trimmed, lowercased)
    Normalized,
    /// Fuzzy / Levenshtein match (similarity > threshold)
    Fuzzy { similarity: f64 },
    /// Could not match — requires manual resolution
    Unresolved,
    /// Newly created — no old map to match against
    New,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceData {
    pub uuid: Uuid,
    pub hash: String,
    /// Normalized hash (whitespace-trimmed, lowercased) for resilient matching
    #[serde(default)]
    pub normalized_hash: String,
    pub text: String,
    pub start_idx: usize,
    /// Article-local sentence index (0-based, sequential within article)
    #[serde(default)]
    pub local_id: u32,
    /// Global sentence ID for cross-article referencing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_sentence_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// How this sentence was resolved during the last parse
    #[serde(default = "default_resolution_method")]
    pub resolution: ResolutionMethod,
}

fn default_resolution_method() -> ResolutionMethod {
    ResolutionMethod::New
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceMap {
    #[serde(rename = "sentence_map")]
    pub map: HashMap<Uuid, SentenceData>,
    /// Diagnostics from the last parse/repair run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<AnchoringDiagnostics>,
}

pub struct SentenceParser;

impl SentenceParser {
    pub fn parse(text: &str, old_map: Option<&SentenceMap>) -> SentenceMap {
        let mut new_map = HashMap::new();
        let sentences = Self::split_into_sentences(text);

        let mut old_hashes: HashMap<String, Uuid> = HashMap::new();
        let mut old_normalized_hashes: HashMap<String, Uuid> = HashMap::new();
        let mut old_texts: Vec<(Uuid, String)> = Vec::new();
        let mut old_data_map: HashMap<Uuid, serde_json::Value> = HashMap::new();
        let mut old_global_ids: HashMap<Uuid, Uuid> = HashMap::new();

        if let Some(om) = old_map {
            for (uuid, data) in &om.map {
                old_hashes.insert(data.hash.clone(), *uuid);
                if !data.normalized_hash.is_empty() {
                    old_normalized_hashes.insert(data.normalized_hash.clone(), *uuid);
                } else {
                    // Compute normalized hash for old data that doesn't have it
                    let nh = Self::compute_normalized_hash(&data.text);
                    old_normalized_hashes.insert(nh, *uuid);
                }
                old_texts.push((*uuid, data.text.clone()));
                if let Some(meta) = &data.metadata {
                    old_data_map.insert(*uuid, meta.clone());
                }
                if let Some(gid) = data.global_sentence_id {
                    old_global_ids.insert(*uuid, gid);
                }
            }
        }

        let mut diag = AnchoringDiagnostics {
            exact_matches: 0,
            normalized_matches: 0,
            fuzzy_matches: 0,
            unresolved: 0,
            total_sentences: sentences.len() as u32,
        };

        for (local_id, (start_idx, sentence_text)) in sentences.into_iter().enumerate() {
            let hash = Self::compute_hash(&sentence_text);
            let normalized_hash = Self::compute_normalized_hash(&sentence_text);

            // 1. Exact Hash Match
            if let Some(uuid) = old_hashes.get(&hash) {
                if !new_map.contains_key(uuid) {
                    new_map.insert(
                        *uuid,
                        SentenceData {
                            uuid: *uuid,
                            hash: hash.clone(),
                            normalized_hash,
                            text: sentence_text,
                            start_idx,
                            local_id: local_id as u32,
                            global_sentence_id: old_global_ids.get(uuid).copied(),
                            metadata: old_data_map.get(uuid).cloned(),
                            resolution: ResolutionMethod::Exact,
                        },
                    );
                    diag.exact_matches += 1;
                    continue;
                }
            }

            // 2. Normalized Hash Match
            if let Some(uuid) = old_normalized_hashes.get(&normalized_hash) {
                if !new_map.contains_key(uuid) {
                    new_map.insert(
                        *uuid,
                        SentenceData {
                            uuid: *uuid,
                            hash: hash.clone(),
                            normalized_hash,
                            text: sentence_text,
                            start_idx,
                            local_id: local_id as u32,
                            global_sentence_id: old_global_ids.get(uuid).copied(),
                            metadata: old_data_map.get(uuid).cloned(),
                            resolution: ResolutionMethod::Normalized,
                        },
                    );
                    diag.normalized_matches += 1;
                    continue;
                }
            }

            // 3. Fuzzy Match (Levenshtein)
            let mut best_match_uuid: Option<Uuid> = None;
            let mut best_match_score = 0.0;

            for (uuid, old_text) in &old_texts {
                if new_map.contains_key(uuid) {
                    continue;
                }

                let distance = levenshtein(&sentence_text, old_text);
                let max_len = sentence_text.len().max(old_text.len());
                if max_len == 0 {
                    continue;
                }

                let similarity = 1.0 - (distance as f64 / max_len as f64);

                if similarity > 0.85 && similarity > best_match_score {
                    best_match_score = similarity;
                    best_match_uuid = Some(*uuid);
                }
            }

            let (final_uuid, resolution) = if let Some(uuid) = best_match_uuid {
                diag.fuzzy_matches += 1;
                (
                    uuid,
                    ResolutionMethod::Fuzzy {
                        similarity: best_match_score,
                    },
                )
            } else if old_map.is_some() {
                // Had old map but couldn't match — unresolved
                diag.unresolved += 1;
                (Uuid::new_v4(), ResolutionMethod::Unresolved)
            } else {
                // No old map — brand new
                (Uuid::new_v4(), ResolutionMethod::New)
            };

            let metadata = if let Some(ref uuid) = best_match_uuid {
                old_data_map.get(uuid).cloned()
            } else {
                None
            };

            let global_sentence_id = best_match_uuid
                .and_then(|uuid| old_global_ids.get(&uuid).copied());

            new_map.insert(
                final_uuid,
                SentenceData {
                    uuid: final_uuid,
                    hash,
                    normalized_hash,
                    text: sentence_text,
                    start_idx,
                    local_id: local_id as u32,
                    global_sentence_id,
                    metadata,
                    resolution,
                },
            );
        }

        SentenceMap {
            map: new_map,
            diagnostics: Some(diag),
        }
    }

    /// Get unresolved sentences that need manual rebinding
    pub fn get_unresolved(map: &SentenceMap) -> Vec<&SentenceData> {
        map.map
            .values()
            .filter(|s| s.resolution == ResolutionMethod::Unresolved)
            .collect()
    }

    fn split_into_sentences(text: &str) -> Vec<(usize, String)> {
        let re = Regex::new(r"([.!?]+)(\s+|$)").unwrap();
        let mut sentences = Vec::new();
        let mut last_idx = 0;

        for cap in re.captures_iter(text) {
            let m = cap.get(0).unwrap();
            let end_idx = m.end();

            let sentence = text[last_idx..end_idx].trim().to_string();
            if !sentence.is_empty() {
                sentences.push((last_idx, sentence));
            }
            last_idx = end_idx;
        }

        // Catch tail
        if last_idx < text.len() {
            let tail = text[last_idx..].trim().to_string();
            if !tail.is_empty() {
                sentences.push((last_idx, tail));
            }
        }

        sentences
    }

    fn compute_hash(text: &str) -> String {
        let digest = md5::compute(text.as_bytes());
        format!("{:x}", digest)
    }

    /// Compute a normalized hash that is resilient to whitespace and case changes
    fn compute_normalized_hash(text: &str) -> String {
        let normalized = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let digest = md5::compute(normalized.as_bytes());
        format!("{:x}", digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_new_text_produces_all_new_resolutions() {
        let text = "Hello world. This is a test. Another sentence.";
        let result = SentenceParser::parse(text, None);

        assert_eq!(result.map.len(), 3);
        for data in result.map.values() {
            assert_eq!(data.resolution, ResolutionMethod::New);
            assert!(!data.hash.is_empty());
            assert!(!data.normalized_hash.is_empty());
        }

        let diag = result.diagnostics.unwrap();
        assert_eq!(diag.total_sentences, 3);
        assert_eq!(diag.exact_matches, 0);
    }

    #[test]
    fn exact_match_preserves_uuid_and_metadata() {
        let text = "Hello world. This is a test.";
        let old = SentenceParser::parse(text, None);

        let old_uuid = old.map.values().find(|s| s.text.contains("Hello")).unwrap().uuid;

        // Re-parse same text
        let new = SentenceParser::parse(text, Some(&old));

        let matched = new.map.get(&old_uuid).expect("Should preserve UUID");
        assert_eq!(matched.resolution, ResolutionMethod::Exact);

        let diag = new.diagnostics.unwrap();
        assert_eq!(diag.exact_matches, 2);
        assert_eq!(diag.unresolved, 0);
    }

    #[test]
    fn normalized_match_handles_whitespace_changes() {
        let text1 = "Hello   world.  This is a test.";
        let old = SentenceParser::parse(text1, None);

        let text2 = "Hello world. This is a test.";
        let new = SentenceParser::parse(text2, Some(&old));

        let diag = new.diagnostics.unwrap();
        // Should get normalized or exact matches for both
        assert_eq!(diag.unresolved, 0);
    }

    #[test]
    fn fuzzy_match_handles_small_edits() {
        let text1 = "The quick brown fox jumps over the lazy dog.";
        let old = SentenceParser::parse(text1, None);

        let text2 = "The quick brown fox jumped over the lazy dog."; // "jumps" → "jumped"
        let new = SentenceParser::parse(text2, Some(&old));

        let diag = new.diagnostics.unwrap();
        assert_eq!(diag.fuzzy_matches, 1);
        assert_eq!(diag.unresolved, 0);
    }

    #[test]
    fn completely_different_sentence_is_unresolved() {
        let text1 = "Hello world.";
        let old = SentenceParser::parse(text1, None);

        let text2 = "Completely different content here.";
        let new = SentenceParser::parse(text2, Some(&old));

        let diag = new.diagnostics.unwrap();
        assert_eq!(diag.unresolved, 1);
    }

    #[test]
    fn get_unresolved_returns_correct_entries() {
        let text1 = "Hello world.";
        let old = SentenceParser::parse(text1, None);

        let text2 = "Completely new sentence.";
        let new = SentenceParser::parse(text2, Some(&old));

        let unresolved = SentenceParser::get_unresolved(&new);
        assert_eq!(unresolved.len(), 1);
        assert_eq!(unresolved[0].text, "Completely new sentence.");
    }

    #[test]
    fn local_ids_are_sequential() {
        let text = "First. Second. Third.";
        let result = SentenceParser::parse(text, None);

        let mut entries: Vec<_> = result.map.values().collect();
        entries.sort_by_key(|e| e.local_id);

        assert_eq!(entries[0].local_id, 0);
        assert_eq!(entries[1].local_id, 1);
        assert_eq!(entries[2].local_id, 2);
    }
}
