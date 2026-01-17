use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use regex::Regex;
use md5::{Md5, Digest};
use strsim::levenshtein;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceData {
    pub uuid: Uuid,
    pub hash: String,
    pub text: String,
    pub start_idx: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceMap {
    pub map: HashMap<Uuid, SentenceData>,
}

pub struct SentenceParser;

impl SentenceParser {
    pub fn parse(text: &str, old_map: Option<&SentenceMap>) -> SentenceMap {
        let mut new_map = HashMap::new();
        let sentences = Self::split_into_sentences(text);
        
        let mut old_hashes: HashMap<String, Uuid> = HashMap::new();
        let mut old_texts: Vec<(Uuid, String)> = Vec::new();

        if let Some(om) = old_map {
            for (uuid, data) in &om.map {
                old_hashes.insert(data.hash.clone(), *uuid);
                old_texts.push((*uuid, data.text.clone()));
            }
        }

        for (start_idx, sentence_text) in sentences {
            let hash = Self::compute_hash(&sentence_text);
            
            // 1. Exact Match
            if let Some(uuid) = old_hashes.get(&hash) {
                new_map.insert(*uuid, SentenceData {
                    uuid: *uuid,
                    hash: hash.clone(),
                    text: sentence_text,
                    start_idx,
                });
                continue;
            }

            // 2. Fuzzy Match (Hybrid Anchoring)
            let mut best_match_uuid: Option<Uuid> = None;
            let mut best_match_score = 0.0;

            for (uuid, old_text) in &old_texts {
                // Skip if already reused
                if new_map.contains_key(uuid) {
                    continue;
                }

                let distance = levenshtein(&sentence_text, old_text);
                let max_len = sentence_text.len().max(old_text.len());
                if max_len == 0 { continue; }
                
                let similarity = 1.0 - (distance as f64 / max_len as f64);
                
                // Threshold > 0.85
                if similarity > 0.85 && similarity > best_match_score {
                    best_match_score = similarity;
                    best_match_uuid = Some(*uuid);
                }
            }

            let final_uuid = if let Some(uuid) = best_match_uuid {
                uuid
            } else {
                Uuid::new_v4()
            };

            new_map.insert(final_uuid, SentenceData {
                uuid: final_uuid,
                hash,
                text: sentence_text,
                start_idx,
            });
        }

        SentenceMap { map: new_map }
    }

    fn split_into_sentences(text: &str) -> Vec<(usize, String)> {
        // Naive Regex Splitter for MVP
        // Matches [.!?] followed by space or EOF, avoiding Mr. Mrs. etc via lookbehind if possible
        // Rust regex doesn't support lookbehind/lookaround nicely, so we iterate
        
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
        let mut hasher = Md5::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
