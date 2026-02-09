use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use fst::{Set, IntoStreamer, Streamer};
use fst::automaton::Levenshtein;
use std::process::Command;

#[derive(Clone)]
pub struct DictionaryLoader {
    dicts: Arc<Vec<Mutex<CustomDict>>>,
    index: Arc<Set<Vec<u8>>>,
}

struct CustomDict {
    #[allow(dead_code)]
    name: String,
    file_path: PathBuf,
    // Map word -> (offset, size)
    index_map: std::collections::HashMap<String, (u64, u64)>, 
    file: Option<File>,
}

impl CustomDict {
    fn new(base_path: PathBuf) -> Option<Self> {
        let name = base_path.file_stem()?.to_string_lossy().to_string();
        let idx_path = base_path.with_extension("idx");
        let dict_path = base_path.with_extension("dict");
        let dz_path = base_path.with_extension("dict.dz");

        // 1. Parse Index
        if !idx_path.exists() { return None; }
        
        let mut index_map = std::collections::HashMap::new();
        // Use the parsing logic
        if let Ok(entries) = parse_idx_entries(&idx_path) {
            for (word, offset, size) in entries {
                index_map.insert(word, (offset, size));
            }
        } else {
            return None; 
        }

        // 2. Handle Dict File
        let final_dict_path = if dict_path.exists() {
            dict_path
        } else if dz_path.exists() {
            // Try to decompress
            tracing::info!("Decompressing {}...", dz_path.display());
            // gzip -dk -S .dz (keep original, specify suffix)
            let status = Command::new("gzip")
                .arg("-dk")
                .arg("-S")
                .arg(".dz")
                .arg(&dz_path)
                .status();
            
            if status.is_err() || !status.unwrap().success() {
                tracing::error!("Failed to decompress {}", dz_path.display());
                // We can't use this dict
                return None;
            }
            dict_path // Now it should exist
        } else {
            return None;
        };

        Some(Self {
            name,
            file_path: final_dict_path,
            index_map,
            file: None, // Lazy open
        })
    }
    
    fn get(&mut self, word: &str) -> Option<String> {
        if let Some(&(offset, size)) = self.index_map.get(word) {
            if self.file.is_none() {
                self.file = File::open(&self.file_path).ok();
            }
            
            if let Some(file) = &mut self.file {
                if file.seek(SeekFrom::Start(offset)).is_ok() {
                    let mut buffer = vec![0u8; size as usize];
                    if file.read_exact(&mut buffer).is_ok() {
                        return String::from_utf8(buffer).ok();
                    }
                }
            }
        }
        None
    }
}

// Helper to parse .idx file
fn parse_idx_entries(idx_path: &Path) -> Result<Vec<(String, u64, u64)>, std::io::Error> {
    let mut file = File::open(idx_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let mut entries = Vec::new();
    let mut cursor = 0;
    
    while cursor < buffer.len() {
         let start = cursor;
         while cursor < buffer.len() && buffer[cursor] != 0 {
             cursor += 1;
         }
         if cursor >= buffer.len() { break; }
         
         let word_bytes = buffer[start..cursor].to_vec();
         cursor += 1; // Skip null
         
         if let Ok(word) = String::from_utf8(word_bytes) {
             if cursor + 8 <= buffer.len() {
                // Read Big Endian (Network Byte Order) usually for StarDict? 
                // Spec says: "The byte order of the number in .idx file is Network Byte Order (Big-Endian)."
                
                // Read Offset (4 bytes)
                let offset_bytes: [u8; 4] = buffer[cursor..cursor+4].try_into().unwrap();
                let offset = u32::from_be_bytes(offset_bytes) as u64;
                cursor += 4;
                
                // Read Size (4 bytes)
                let size_bytes: [u8; 4] = buffer[cursor..cursor+4].try_into().unwrap();
                let size = u32::from_be_bytes(size_bytes) as u64;
                cursor += 4;
                
                entries.push((word, offset, size));
             } else {
                 break; 
             }
         } else {
             // Invalid utf8 word, skip or break
             cursor += 8; 
         }
    }
    
    Ok(entries)
}


impl DictionaryLoader {
    pub fn new(base_path: &str) -> Self {
        tracing::info!("Initializing DictionaryLoader from: {}", base_path);
        let mut dicts = Vec::new();
        let mut all_words = std::collections::BTreeSet::new();

        let path = Path::new(base_path);
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(sub_entries) = std::fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                             let sub_path = sub_entry.path();
                             if let Some(ext) = sub_path.extension() {
                                 if ext == "ifo" {
                                     let base = sub_path.with_extension("");
                                     if let Some(d) = CustomDict::new(base) {
                                         // Collect words for FST
                                         for word in d.index_map.keys() {
                                             all_words.insert(word.as_bytes().to_vec());
                                         }
                                         dicts.push(Mutex::new(d));
                                     }
                                 }
                             }
                        }
                    }
                }
            }
        }

        tracing::info!("Building FST Index with {} words...", all_words.len());
        let index = Set::from_iter(all_words).unwrap_or_else(|_| Set::from_iter(Vec::<Vec<u8>>::new()).unwrap());

        Self {
            dicts: Arc::new(dicts),
            index: Arc::new(index),
        }
    }

    pub fn lookup(&self, word: &str) -> Option<Vec<String>> {
        let mut results = Vec::new();
        
        for dict_mutex in self.dicts.iter() {
             if let Ok(mut dict) = dict_mutex.lock() {
                 if let Some(def) = dict.get(word) {
                     results.push(def);
                 }
             }
        }
        
        if results.is_empty() {
             None
        } else {
             Some(results)
        }
    }

    pub async fn fuzzy_search(&self, word: &str) -> Vec<String> {
        let index = self.index.clone();
        let query = word.to_string();

        tokio::task::spawn_blocking(move || {
            let dist = if query.len() < 4 { 1 } else { 2 };
            
            // 1. Always check for exact match first
            let mut matches = Vec::new();
            if index.contains(query.as_bytes()) {
                 matches.push(query.clone());
            }

            let lev = match Levenshtein::new(&query, dist) {
                 Ok(l) => l,
                 Err(_) => return matches
            };

            let mut stream = index.search(lev).into_stream();
            
            // Collect more candidates (e.g. 100) to allow for sorting
            while let Some(key) = stream.next() {
                if let Ok(s) = String::from_utf8(key.to_vec()) {
                    // Avoid duplicate if exact match was already added
                    if s != query {
                        matches.push(s);
                    }
                }
                if matches.len() >= 100 { break; } 
            }

            // 2. Sort by Levenshtein Distance (Quality)
            // We need to re-calculate distance because FST stream doesn't give it
            matches.sort_by(|a, b| {
                let dist_a = strsim::levenshtein(a, &query);
                let dist_b = strsim::levenshtein(b, &query);
                
                if dist_a != dist_b {
                    dist_a.cmp(&dist_b)
                } else {
                    // Secondary sort: Length (shorter is better?), then Alphabetical
                    if a.len() != b.len() {
                        a.len().cmp(&b.len())
                    } else {
                        a.cmp(b)
                    }
                }
            });

            // Return top 20 after sorting
            matches.into_iter().take(20).collect()
        }).await.unwrap_or_default()
    }
}
