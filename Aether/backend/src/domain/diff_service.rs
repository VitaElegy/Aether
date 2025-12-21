use similar::{ChangeTag, TextDiff};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffResult {
    pub changes: Vec<DiffChange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffChange {
    pub tag: String, // "Equal", "Delete", "Insert"
    pub value: String,
}

pub struct DiffService;

impl DiffService {
    pub fn compute_diff(old_text: &str, new_text: &str) -> DiffResult {
        let diff = TextDiff::from_lines(old_text, new_text);
        let mut changes = Vec::new();

        for change in diff.iter_all_changes() {
            let tag = match change.tag() {
                ChangeTag::Delete => "Delete".to_string(),
                ChangeTag::Insert => "Insert".to_string(),
                ChangeTag::Equal => "Equal".to_string(),
            };
            changes.push(DiffChange {
                tag,
                value: change.to_string(),
            });
        }

        DiffResult { changes }
    }
}

