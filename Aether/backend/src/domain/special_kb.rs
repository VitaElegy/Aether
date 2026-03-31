pub const DEFAULT_RENDERER_ID: &str = "default";
pub const ASSETS_RENDERER_ID: &str = "assets_v1";
pub const ADMIN_SYSTEM_RENDERER_ID: &str = "admin_system";

fn normalize_lookup_key(renderer_id: &str) -> Option<String> {
    let normalized = renderer_id
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_lowercase();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub fn normalize_renderer_id(renderer_id: Option<&str>) -> Option<String> {
    let normalized = normalize_lookup_key(renderer_id?)?;

    let canonical = match normalized.as_str() {
        "knowledge" => DEFAULT_RENDERER_ID,
        "memo_std" | "memo_v1" => "memo",
        "vocabulary_std" => "vocabulary",
        "english" | "english_v1_std" => "english_v1",
        "article_analysis" | "english_analysis" | "english analysis" => "article-analysis",
        "math" | "math_std" => "math_v3",
        "math_v1_std" => "math_v1",
        "vrkb_std" | "vulnerability_research" => "vrkb",
        "assets" => ASSETS_RENDERER_ID,
        "admin" | "system" => ADMIN_SYSTEM_RENDERER_ID,
        _ => normalized.as_str(),
    };

    Some(canonical.to_string())
}

pub fn renderer_id_or_default(renderer_id: Option<&str>) -> String {
    normalize_renderer_id(renderer_id).unwrap_or_else(|| DEFAULT_RENDERER_ID.to_string())
}

pub fn is_assets_renderer(renderer_id: Option<&str>) -> bool {
    normalize_renderer_id(renderer_id).as_deref() == Some(ASSETS_RENDERER_ID)
}

#[cfg(test)]
mod tests {
    use super::{
        is_assets_renderer, normalize_renderer_id, renderer_id_or_default,
        ADMIN_SYSTEM_RENDERER_ID, ASSETS_RENDERER_ID, DEFAULT_RENDERER_ID,
    };

    #[test]
    fn normalizes_legacy_renderer_aliases_to_canonical_values() {
        assert_eq!(
            normalize_renderer_id(Some("knowledge")).as_deref(),
            Some(DEFAULT_RENDERER_ID)
        );
        assert_eq!(
            normalize_renderer_id(Some(" memo_std ")).as_deref(),
            Some("memo")
        );
        assert_eq!(
            normalize_renderer_id(Some("ENGLISH")).as_deref(),
            Some("english_v1")
        );
        assert_eq!(
            normalize_renderer_id(Some("english analysis")).as_deref(),
            Some("article-analysis"),
        );
        assert_eq!(
            normalize_renderer_id(Some("math")).as_deref(),
            Some("math_v3")
        );
        assert_eq!(
            normalize_renderer_id(Some("vulnerability_research")).as_deref(),
            Some("vrkb"),
        );
        assert_eq!(
            normalize_renderer_id(Some("assets")).as_deref(),
            Some(ASSETS_RENDERER_ID)
        );
        assert_eq!(
            normalize_renderer_id(Some("ADMIN")).as_deref(),
            Some(ADMIN_SYSTEM_RENDERER_ID)
        );
    }

    #[test]
    fn preserves_canonical_renderer_ids_and_defaults_when_missing() {
        assert_eq!(
            normalize_renderer_id(Some("assets_v1")).as_deref(),
            Some(ASSETS_RENDERER_ID)
        );
        assert_eq!(normalize_renderer_id(Some("prkb")).as_deref(), Some("prkb"));
        assert_eq!(normalize_renderer_id(Some("  ")), None);
        assert_eq!(renderer_id_or_default(None), DEFAULT_RENDERER_ID);
        assert_eq!(
            renderer_id_or_default(Some("knowledge")),
            DEFAULT_RENDERER_ID
        );
    }

    #[test]
    fn detects_assets_renderer_from_aliases() {
        assert!(is_assets_renderer(Some("assets")));
        assert!(is_assets_renderer(Some("assets_v1")));
        assert!(!is_assets_renderer(Some("memo")));
        assert!(!is_assets_renderer(None));
    }
}
