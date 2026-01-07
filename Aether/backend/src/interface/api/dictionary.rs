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
    let matches = state.dictionary.fuzzy_search(&params.word);
    Json(matches)
}

async fn lookup_word(
    State(state): State<AppState>,
    Query(params): Query<LookupRequest>,
) -> impl IntoResponse {
    let word = params.word;
    
    // 0. Local StarDict
    let mut local_entry: Option<DictionaryEntry> = None;
    if let Some(def_str) = state.dictionary.lookup(&word) {
        // StarDict returns raw text, usually formatted or just text.
        // We'll treat it as a generic definition.
        // TODO: Better parsing of StarDict content (which might be HTML or XDXF).
        // For now, simple text wrapping.
        local_entry = Some(DictionaryEntry {
            word: word.clone(),
            phonetic: None,
            meanings: vec![Meaning {
                part_of_speech: "dictionary".to_string(),
                definitions: vec![Definition {
                    definition: def_str,
                    example: None,
                }],
            }],
            translation: None,
            source: "Local StarDict".to_string(),
        });
    }

    // 1. FreeDictionaryAPI (Primary - Definitions)
    // 2. Datamuse (Secondary - Definitions/Phonetics)
    // 3. MyMemory (Translation)

    let mut primary_entry: Option<DictionaryEntry> = None;
    let mut datamuse_entry: Option<DictionaryEntry> = None;

    // 1. FreeDictionaryAPI
    let fd_url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word);
    if let Ok(response) = reqwest::get(&fd_url).await {
        if response.status().is_success() {
            if let Ok(entries) = response.json::<Vec<serde_json::Value>>().await {
                if let Some(first) = entries.first() {
                    primary_entry = Some(map_free_dictionary_to_entry(first.clone()));
                }
            }
        }
    }

    // 2. Datamuse (for fallback or supplementary)
    let dm_url = format!("https://api.datamuse.com/words?sp={}&md=dr&max=1", word);
    if let Ok(response) = reqwest::get(&dm_url).await {
         if response.status().is_success() {
            if let Ok(entries) = response.json::<Vec<serde_json::Value>>().await {
                if let Some(first) = entries.first() {
                    datamuse_entry = Some(map_datamuse_to_entry(first.clone(), &word));
                }
            }
         }
    }

    // 3. Translation (MyMemory)
    let translation = fetch_translation(&word).await;

    // Aggregation Logic
    // If we have local entry, we might prioritize it or merge it.
    // Spec says: "Base: StarDict... Extension: User adds...".
    // We'll treat Local as highly trusted.
    
    let mut final_entry = if let Some(mut local) = local_entry {
        // If we also found online data, we can merge phonetics or extra definitions
        if let Some(p) = primary_entry {
             if local.phonetic.is_none() { local.phonetic = p.phonetic; }
             // Merge definitions? Maybe append online ones.
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
        (StatusCode::NOT_FOUND, Json(final_entry))
    } else {
        (StatusCode::OK, Json(final_entry))
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
