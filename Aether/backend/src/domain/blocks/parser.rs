use super::models::Block;
use uuid::Uuid;
use serde_json::json;
use chrono::Utc;

pub fn parse_markdown_to_blocks(document_id: Uuid, markdown: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = markdown.lines().collect();
    let mut i = 0;
    
    let mut ordinal = 0;

    while i < lines.len() {
        let line = lines[i];
        
        // 1. Heading
        if line.starts_with("#") {
            let level = line.chars().take_while(|c| *c == '#').count();
            let text = line[level..].trim();
            blocks.push(create_block(document_id, "heading", ordinal, json!({
                "level": level,
                "text": text,
                "text_mirror": text // Searchable trait fallback
            })));
            ordinal += 1;
            i += 1;
            continue;
        }

        // 2. Math Block ($$ ... $$)
        if line.trim() == "$$" {
            let mut j = i + 1;
            let mut content = String::new();
            while j < lines.len() {
                 if lines[j].trim() == "$$" {
                     break;
                 }
                 content.push_str(lines[j]);
                 content.push('\n');
                 j += 1;
            }
            // If strictly $$ ... $$
            blocks.push(create_block(document_id, "math", ordinal, json!({
                "latex": content.trim(),
                "text_mirror": content.trim()
            })));
            ordinal += 1;
            i = j + 1;
            continue;
        }

        // 3. Code Block (``` ... ```)
        if line.trim().starts_with("```") {
            let lang = line.trim().trim_start_matches("```").trim();
            let mut j = i + 1;
            let mut content = String::new();
            while j < lines.len() {
                if lines[j].trim().starts_with("```") {
                    break;
                }
                content.push_str(lines[j]);
                content.push('\n');
                j += 1;
            }
            blocks.push(create_block(document_id, "code", ordinal, json!({
                "language": lang,
                "code": content.trim(),
                "text_mirror": content.trim() 
            })));
            ordinal += 1;
            i = j + 1;
            continue;
        }

        // 4. Paragraph (Empty lines separate paragraphs)
        if line.trim().is_empty() {
            i += 1;
            continue;
        }
        
        // Accumulate paragraph
        let mut j = i;
        let mut content = String::new();
        while j < lines.len() {
            let l = lines[j];
            if l.trim().is_empty() || l.starts_with("#") || l.trim() == "$$" || l.trim().starts_with("```") {
                break;
            }
            content.push_str(l);
            content.push('\n');
            j += 1;
        }
        
        blocks.push(create_block(document_id, "paragraph", ordinal, json!({
            "markdown": content.trim(),
            "text_mirror": content.trim()
        })));
        ordinal += 1;
        i = j;
    }

    blocks
}

fn create_block(doc_id: Uuid, type_name: &str, ordinal: i32, payload: serde_json::Value) -> Block {
    Block {
        id: Uuid::new_v4(),
        document_id: doc_id,
        type_name: type_name.to_string(),
        ordinal,
        revision: 1,
        payload,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
