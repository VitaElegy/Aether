use crate::interface::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

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

    // 1. Check Cache (modified to expect Vec<DictionaryEntry>)
    if let Some(cached_json) = state.dictionary_cache.get(&word).await {
        if let Ok(entries) = serde_json::from_str::<Vec<DictionaryEntry>>(&cached_json) {
            return (StatusCode::OK, Json(entries)).into_response();
        }
    }

    let mut all_entries = Vec::new();

    // 0. Local StarDict (Returns Vec<(Source, Definition)>)
    let local_results = state.dictionary.lookup(&word);
    for (source_name, raw_text) in local_results {
        let mut meanings = Vec::new();
        let mut phonetic = None;

        // Heuristic Parsing
        // 1. Extract Phonetic: /.../
        if let Some(start) = raw_text.find('/') {
            if let Some(end) = raw_text[start + 1..].find('/') {
                let p = &raw_text[start..=start + 1 + end];
                if p.len() < 50 {
                    phonetic = Some(p.to_string());
                }
            }
        }

        // 2. Clean text for display
        let mut cleaned = raw_text.clone();
        cleaned = cleaned.replace(" * ", "\n* ");
        for i in 1..10 {
            cleaned = cleaned.replace(&format!(" {} ", i), &format!("\n{}. ", i));
        }
        for c in 'a'..'z' {
            cleaned = cleaned.replace(&format!(" ({}) ", c), &format!("\n({}) ", c));
        }

        meanings.push(Meaning {
            part_of_speech: "Definition".to_string(),
            definitions: vec![Definition {
                definition: cleaned,
                example: None,
            }],
        });

        all_entries.push(DictionaryEntry {
            word: word.clone(),
            phonetic,
            meanings,
            translation: None,
            source: source_name.replace('_', " "), // Format "oxford_advanced" -> "oxford advanced"
        });
    }

    // 1. External APIs (Concurrent)
    let fd_url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word);
    let dm_url = format!("https://api.datamuse.com/words?sp={}&md=dr&max=1", word);

    // Timeout: 1500ms
    let external_task = async {
        tokio::join!(
            reqwest::get(&fd_url),
            reqwest::get(&dm_url),
            fetch_translation(&word) // We still fetch translation, maybe attach to the first entry or a generic one?
        )
    };

    let (fd_opt, dm_opt, translation) =
        match tokio::time::timeout(std::time::Duration::from_millis(1500), external_task).await {
            Ok((fd_res, dm_res, trans)) => (fd_res.ok(), dm_res.ok(), trans),
            Err(_) => {
                tracing::warn!("External dictionary API timed out for '{}'", word);
                (None, None, None)
            }
        };

    // Process FreeDictionaryAPI
    if let Some(response) = fd_opt {
        if response.status().is_success() {
            if let Ok(entries) = response.json::<Vec<serde_json::Value>>().await {
                // FreeDict returns multiple entries (e.g. noun entry, verb entry, or homonyms).
                // We should map ALL of them.
                for entry in entries {
                    all_entries.push(map_free_dictionary_to_entry(entry));
                }
            }
        }
    }

    // Process Datamuse
    if let Some(response) = dm_opt {
        if response.status().is_success() {
            if let Ok(entries) = response.json::<Vec<serde_json::Value>>().await {
                if let Some(first) = entries.first() {
                    all_entries.push(map_datamuse_to_entry(first.clone(), &word));
                }
            }
        }
    }

    // Process Translation (Attach to the best entry or create new one)
    if let Some(t) = translation {
        // If we have entries, attach translation to the first one?
        // Or maybe create a dedicated "Translator" entry?
        // Let's create a dedicated entry for clarity if it's purely translation.
        // Actually, users might prefer it on the main entry.
        // Let's attach to the first entry if available, OTHERWISE create a "MyMemory" entry.
        if let Some(first) = all_entries.first_mut() {
            if first.translation.is_none() {
                first.translation = Some(t);
                // We don't change the source name here, just enrich it.
            }
        } else {
            all_entries.push(DictionaryEntry {
                word: word.clone(),
                phonetic: None,
                meanings: vec![],
                translation: Some(t),
                source: "MyMemory".to_string(),
            });
        }
    }

    if all_entries.is_empty() {
        // Return 404 but with empty list? Or just empty list?
        // Frontend expects a list now.
        // Previously we returned 404 with a dummy entry.
        // Let's return 200 OK with empty list, frontend handles "No definition found".
        (StatusCode::OK, Json(Vec::<DictionaryEntry>::new())).into_response()
    } else {
        // Cache the result
        if let Ok(json_str) = serde_json::to_string(&all_entries) {
            state.dictionary_cache.insert(word, json_str).await;
        }
        (StatusCode::OK, Json(all_entries)).into_response()
    }
}

async fn fetch_translation(word: &str) -> Option<String> {
    // MyMemory API: https://api.mymemory.translated.net/get?q=Hello World&langpair=en|zh
    let url = format!(
        "https://api.mymemory.translated.net/get?q={}&langpair=en|zh",
        word
    );

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
    let phonetic = raw["phonetic"]
        .as_str()
        .map(|s| s.to_string())
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
                    }
                    .to_string();

                    let def_text = parts[1].to_string();

                    if let Some(existing_meaning) = meanings_list
                        .iter_mut()
                        .find(|m: &&mut Meaning| m.part_of_speech == pos)
                    {
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
