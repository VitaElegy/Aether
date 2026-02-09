use axum::{
    Router,
    routing::get,
    extract::{Query, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::interface::state::AppState;

#[derive(Deserialize)]
pub struct LookupRequest {
    pub word: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DictionaryEntry {
    pub word: String,
    pub phonetic: Option<String>,
    pub meanings: Vec<Meaning>,
    pub translation: Option<String>,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meaning {
    #[serde(rename = "partOfSpeech")]
    pub part_of_speech: String,
    pub definitions: Vec<Definition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Definition {
    pub definition: String,
    pub example: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/dictionary/lookup", get(lookup_word))
        .route("/api/dictionary/fuzzy", get(fuzzy_search))
}

async fn fuzzy_search(
    State(state): State<AppState>,
    Query(params): Query<LookupRequest>,
) -> impl IntoResponse {
    let matches = state.dictionary.fuzzy_search(&params.word).await;
    Json(matches)
}

async fn lookup_word(
    State(state): State<AppState>,
    Query(params): Query<LookupRequest>,
) -> impl IntoResponse {
    let word = params.word;

    // 1. Check Cache
    if let Some(cached_json) = state.dictionary_cache.get(&word).await {
        if let Ok(entry) = serde_json::from_str::<DictionaryEntry>(&cached_json) {
            return (StatusCode::OK, Json(entry)).into_response();
        }
    }
    
    // 0. Local StarDict
    let mut local_entry: Option<DictionaryEntry> = None;
    if let Some(raw_definitions) = state.dictionary.lookup(&word) {
        let mut meanings = Vec::new();
        let mut phonetic = None;
        let mut current_source = "Local StarDict".to_string();

        for raw_text in raw_definitions {
            // Check if this is likely Oxford by looking for specific patterns or just general parsing
            // Heuristic Parsing for complex formats like Oxford:
            // 1. Extract Phonetic: /.../
            if let Some(start) = raw_text.find('/') {
                if let Some(end) = raw_text[start+1..].find('/') {
                    let p = &raw_text[start..=start+1+end];
                    // Basic validation to ensure it's not just a slash in text
                    if p.len() < 50 { 
                         phonetic = Some(p.to_string());
                    }
                }
            }

            // 2. Identify common POS markers to split main blocks
            // Oxford often doesn't use newlines, just spaces. e.g. "word... n 1 [C]..."
            // We need a Regex or smart split. Regex is cleaner but requires dependency.
            // Let's use a manual state machine for stability without adding generic regex dep if possible,
            // or just add `regex` crate which is standard. Using regex is safer for this complexity.
            
            // For now, let's try to split by known POS tags if they are surrounded by spaces/start
            // Tags: " n ", " v ", " adj ", " adv ", " prep ", " conj "
            // Also numbered lists: " 1 ", " 2 "
            
            // SIMPLIFIED STRATEGY: 
            // 1. Split by numbers " 1 ", " 2 " to get main definitions.
            // 2. Inside each, look for (a), (b).
            
            // Let's treat the whole text as one block if we can't easily split POS, 
            // but we can try to improve the display by inserting newlines for the frontend parser.
            
            // ACTUALLY, the user wants "Clear Separation".
            // Let's clean the text: 
            // Replace " * " with "\n* " (New bullet point)
            // Replace " 1 " with "\n1. "
            // Replace " 2 " with "\n2. "
            // Replace "(a)" with "\n(a)"
            
            let mut cleaned = raw_text.clone();
            cleaned = cleaned.replace(" * ", "\n* ");
            
            // Insert breaks before numbers 1-9
            for i in 1..10 {
                cleaned = cleaned.replace(&format!(" {} ", i), &format!("\n{}. ", i));
            }
            
            // Insert breaks for (a), (b)...
            for c in 'a'..'z' {
                 cleaned = cleaned.replace(&format!(" ({}) ", c), &format!("\n({}) ", c));
            }

            // Try to set source name if possible (heuristic)
            if cleaned.contains("Oxford") {
                current_source = "Oxford Dictionary".to_string();
            }

            // Create a generic meaning block with this formatted text
            // The frontend will rendering newlines as separate blocks
            meanings.push(Meaning {
                part_of_speech: "Definition".to_string(), 
                definitions: vec![Definition {
                    definition: cleaned,
                    example: None
                }],
            });
        }

        local_entry = Some(DictionaryEntry {
            word: word.clone(),
            phonetic, 
            meanings, 
            translation: None,
            source: current_source,
        });
    }

    // 1. FreeDictionaryAPI, 2. Datamuse, 3. MyMemory
    let mut primary_entry: Option<DictionaryEntry> = None;
    let mut datamuse_entry: Option<DictionaryEntry> = None;

    let fd_url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word);
    let dm_url = format!("https://api.datamuse.com/words?sp={}&md=dr&max=1", word);

    // Timeout: 1500ms. If external APIs are slow, we fallback to Local or None.
    let external_task = async {
        tokio::join!(
            reqwest::get(&fd_url),
            reqwest::get(&dm_url),
            fetch_translation(&word)
        )
    };

    // Timeout: 1500ms.
    let (fd_opt, dm_opt, translation) = match tokio::time::timeout(std::time::Duration::from_millis(1500), external_task).await {
        Ok((fd_res, dm_res, trans)) => (fd_res.ok(), dm_res.ok(), trans),
        Err(_) => {
            tracing::warn!("External dictionary API timed out for '{}'", word);
            (None, None, None)
        }
    };

    // 1. Process FreeDictionaryAPI
    if let Some(response) = fd_opt {
        if response.status().is_success() {
            if let Ok(entries) = response.json::<Vec<serde_json::Value>>().await {
                if let Some(first) = entries.first() {
                    primary_entry = Some(map_free_dictionary_to_entry(first.clone()));
                }
            }
        }
    }

    // 2. Process Datamuse
    if let Some(response) = dm_opt {
         if response.status().is_success() {
            if let Ok(entries) = response.json::<Vec<serde_json::Value>>().await {
                if let Some(first) = entries.first() {
                    datamuse_entry = Some(map_datamuse_to_entry(first.clone(), &word));
                }
            }
         }
    }

    // Aggregation Logic
    let mut final_entry = if let Some(mut local) = local_entry {
        if let Some(p) = primary_entry {
             if local.phonetic.is_none() { local.phonetic = p.phonetic; }
             local.source = format!("{}, {}", local.source, p.source);
             local.meanings.extend(p.meanings);
        } else if let Some(d) = datamuse_entry {
              if local.phonetic.is_none() { local.phonetic = d.phonetic; }
               local.source = format!("{}, {}", local.source, d.source);
               local.meanings.extend(d.meanings);
        }
        local
    } else {
        match (primary_entry, datamuse_entry) {
            (Some(mut p), Some(d)) => {
                p.source = format!("{}, Datamuse", p.source);
                for m in d.meanings {
                     p.meanings.push(m);
                }
                p
            },
            (Some(p), None) => p,
            (None, Some(d)) => d,
            (None, None) => DictionaryEntry {
                word: word.clone(),
                phonetic: None,
                meanings: vec![],
                translation: None,
                source: "None".to_string(),
            },
        }
    };

    if let Some(t) = translation {
        final_entry.translation = Some(t);
        if final_entry.source == "None" {
             final_entry.source = "MyMemory".to_string();
        } else {
             final_entry.source = format!("{}, MyMemory", final_entry.source);
        }
    }

    if final_entry.source == "None" {
        (StatusCode::NOT_FOUND, Json(final_entry)).into_response()
    } else {
        // Cache the result
        if let Ok(json_str) = serde_json::to_string(&final_entry) {
            state.dictionary_cache.insert(word, json_str).await;
        }
        (StatusCode::OK, Json(final_entry)).into_response()
    }
}

async fn fetch_translation(word: &str) -> Option<String> {
    // MyMemory API: https://api.mymemory.translated.net/get?q=Hello World&langpair=en|zh
    let url = format!("https://api.mymemory.translated.net/get?q={}&langpair=en|zh", word);
    
    if let Ok(response) = reqwest::get(&url).await {
        if response.status().is_success() {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                 if let Some(matches) = json["matches"].as_array() {
                     // Try to find a good quality match, or just take the first one
                     for m in matches {
                         // Some logic to filter could go here
                         if let Some(trans) = m["translation"].as_str() {
                             return Some(trans.to_string());
                         }
                     }
                 }
                 // Fallback to responseData.translatedText
                 if let Some(text) = json["responseData"]["translatedText"].as_str() {
                     return Some(text.to_string());
                 }
            }
        }
    }
    None
}

fn map_free_dictionary_to_entry(raw: serde_json::Value) -> DictionaryEntry {
    let word = raw["word"].as_str().unwrap_or("").to_string();
    let phonetic = raw["phonetic"].as_str().map(|s| s.to_string())
        .or_else(|| raw["phonetics"][0]["text"].as_str().map(|s| s.to_string()));
    
    let mut meanings_list = Vec::new();
    if let Some(meanings) = raw["meanings"].as_array() {
        for m in meanings {
            let pos = m["partOfSpeech"].as_str().unwrap_or("unknown").to_string();
            let mut defs_list = Vec::new();
            if let Some(defs) = m["definitions"].as_array() {
                for d in defs {
                     defs_list.push(Definition {
                         definition: d["definition"].as_str().unwrap_or("").to_string(),
                         example: d["example"].as_str().map(|s| s.to_string()),
                     });
                }
            }
            meanings_list.push(Meaning {
                part_of_speech: pos,
                definitions: defs_list,
            });
        }
    }

    DictionaryEntry {
        word,
        phonetic,
        meanings: meanings_list,
        translation: None, // Will be filled later
        source: "FreeDictionaryAPI".to_string(),
    }
}

fn map_datamuse_to_entry(raw: serde_json::Value, word: &str) -> DictionaryEntry {
    // Datamuse returns: [{"word":"foo","score":123,"defs":["n\tA generic term..."],"tags":["ipa_..."]}]
    // We used md=dr so accessing "defs" and "tags"
    
    let phonetic = if let Some(tags) = raw["tags"].as_array() {
        tags.iter()
            .filter_map(|t| t.as_str())
            .find(|t| t.starts_with("ipa_pron:"))
            .map(|t| t.replace("ipa_pron:", ""))
    } else {
        None
    };

    let mut meanings_list = Vec::new();
    
    if let Some(defs) = raw["defs"].as_array() {
        // format: "part_of_speech<TAB>definition"
        // e.g. "n\tThe top layer..."
        for d_val in defs {
            if let Some(d_str) = d_val.as_str() {
                let parts: Vec<&str> = d_str.splitn(2, '\t').collect();
                if parts.len() == 2 {
                    let pos = match parts[0] {
                        "n" => "noun",
                        "v" => "verb",
                        "adj" => "adjective",
                        "adv" => "adverb",
                        "u" => "unknown",
                        o => o,
                    }.to_string();
                    
                    let def_text = parts[1].to_string();
                    
                    if let Some(existing_meaning) = meanings_list.iter_mut().find(|m: &&mut Meaning| m.part_of_speech == pos) {
                        existing_meaning.definitions.push(Definition {
                            definition: def_text,
                            example: None,
                        });
                    } else {
                        meanings_list.push(Meaning {
                            part_of_speech: pos,
                            definitions: vec![Definition {
                                definition: def_text,
                                example: None,
                            }],
                        });
                    }
                }
            }
        }
    }

    DictionaryEntry {
        word: word.to_string(),
        phonetic,
        meanings: meanings_list,
        translation: None,
        source: "Datamuse".to_string(),
    }
}
