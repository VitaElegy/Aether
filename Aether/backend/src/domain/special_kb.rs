pub const DEFAULT_RENDERER_ID: &str = "default";
pub const ASSETS_RENDERER_ID: &str = "assets_v1";
pub const ADMIN_SYSTEM_RENDERER_ID: &str = "admin_system";

/// Canonical English module renderer ID — all English-related identities resolve here.
pub const ENGLISH_RENDERER_ID: &str = "english_v1";

/// English module capabilities that map to frontend tab modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EnglishCapability {
    /// Vocabulary spotlight, library, batch operations
    Vocabulary,
    /// Article inbox, reader, analysis workspace
    ArticleWorkspace,
    /// Search & intelligence pipeline
    Search,
    /// Import / Export portability
    Portability,
}

/// Describes a tab mode contract for the English shell.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct EnglishTabMode {
    pub id: &'static str,
    pub label: &'static str,
    pub capability: EnglishCapability,
    pub default: bool,
}

/// Shell launch rules: how to open the English module from different contexts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct EnglishShellLaunchRule {
    pub source: &'static str,
    pub target_tab: &'static str,
    pub description: &'static str,
}

/// The canonical capability map for the English module.
#[allow(dead_code)]
pub fn english_capability_map() -> Vec<EnglishTabMode> {
    vec![
        EnglishTabMode {
            id: "vocabulary",
            label: "Words",
            capability: EnglishCapability::Vocabulary,
            default: false,
        },
        EnglishTabMode {
            id: "articles",
            label: "Articles",
            capability: EnglishCapability::ArticleWorkspace,
            default: true,
        },
        EnglishTabMode {
            id: "search",
            label: "Search",
            capability: EnglishCapability::Search,
            default: false,
        },
        EnglishTabMode {
            id: "portability",
            label: "Import/Export",
            capability: EnglishCapability::Portability,
            default: false,
        },
    ]
}

/// Shell launch rules define how external contexts open the English module.
#[allow(dead_code)]
pub fn english_shell_launch_rules() -> Vec<EnglishShellLaunchRule> {
    vec![
        EnglishShellLaunchRule {
            source: "kb_open",
            target_tab: "articles",
            description: "Opening an English KB defaults to article workspace",
        },
        EnglishShellLaunchRule {
            source: "vocabulary_direct",
            target_tab: "vocabulary",
            description: "Direct vocabulary link opens the vocabulary tab",
        },
        EnglishShellLaunchRule {
            source: "article_analysis",
            target_tab: "articles",
            description: "Legacy article-analysis renderer opens articles tab",
        },
        EnglishShellLaunchRule {
            source: "search_query",
            target_tab: "search",
            description: "Opening from a search context activates search tab",
        },
    ]
}

/// Returns true if the given renderer_id resolves to any English identity.
#[allow(dead_code)]
pub fn is_english_renderer(renderer_id: Option<&str>) -> bool {
    normalize_renderer_id(renderer_id).as_deref() == Some(ENGLISH_RENDERER_ID)
}

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
        "vocabulary_std" | "vocabulary" => ENGLISH_RENDERER_ID,
        "english" | "english_v1_std" | "english_v1" => ENGLISH_RENDERER_ID,
        "article_analysis" | "english_analysis" | "english analysis" | "article-analysis" => {
            ENGLISH_RENDERER_ID
        }
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
    use super::*;

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
    fn all_english_identities_resolve_to_single_canonical_id() {
        // All legacy English-related renderer IDs must resolve to ENGLISH_RENDERER_ID
        let english_aliases = [
            "english",
            "ENGLISH",
            "english_v1",
            "english_v1_std",
            "vocabulary",
            "vocabulary_std",
            "article_analysis",
            "english_analysis",
            "english analysis",
            "article-analysis",
        ];
        for alias in english_aliases {
            assert_eq!(
                normalize_renderer_id(Some(alias)).as_deref(),
                Some(ENGLISH_RENDERER_ID),
                "Alias '{}' should resolve to '{}'",
                alias,
                ENGLISH_RENDERER_ID
            );
        }
    }

    #[test]
    fn is_english_renderer_detects_all_aliases() {
        assert!(is_english_renderer(Some("english")));
        assert!(is_english_renderer(Some("vocabulary")));
        assert!(is_english_renderer(Some("article-analysis")));
        assert!(is_english_renderer(Some("english_v1")));
        assert!(!is_english_renderer(Some("memo")));
        assert!(!is_english_renderer(None));
    }

    #[test]
    fn capability_map_has_expected_tabs() {
        let map = english_capability_map();
        assert_eq!(map.len(), 4);
        assert!(map.iter().any(|t| t.id == "vocabulary"));
        assert!(map.iter().any(|t| t.id == "articles"));
        assert!(map.iter().any(|t| t.id == "search"));
        assert!(map.iter().any(|t| t.id == "portability"));
        // Only one default
        assert_eq!(map.iter().filter(|t| t.default).count(), 1);
        assert_eq!(
            map.iter().find(|t| t.default).unwrap().id,
            "articles"
        );
    }

    #[test]
    fn shell_launch_rules_target_valid_tabs() {
        let rules = english_shell_launch_rules();
        let valid_tabs: Vec<&str> = english_capability_map().iter().map(|t| t.id).collect();
        for rule in &rules {
            assert!(
                valid_tabs.contains(&rule.target_tab),
                "Launch rule '{}' targets invalid tab '{}'",
                rule.source,
                rule.target_tab
            );
        }
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
