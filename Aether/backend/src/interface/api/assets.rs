use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::domain::models::{Article, ContentBody, ContentItem, KnowledgeBaseId, UserId};
use crate::domain::ports::{ArticleRepository, KnowledgeBaseRepository};
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;

#[derive(serde::Deserialize)]
pub struct AssetQuery {
    context: Option<Uuid>,
}

#[derive(serde::Deserialize)]
pub struct ListAssetsQuery {
    q: Option<String>,
    asset_type: Option<String>,
    limit: Option<u64>,
    offset: Option<u64>,
    sort_by: Option<String>,
}

#[derive(serde::Serialize, Default)]
pub struct AssetStats {
    pub total: u64,
    pub images: u64,
    pub pdfs: u64,
    pub files: u64,
    pub ip_assets: u64,
    pub domain_assets: u64,
    pub credential_stubs: u64,
    pub snippets: u64,
}

#[derive(serde::Serialize)]
pub struct AssetListResponse {
    pub items: Vec<ContentItem>,
    pub stats: AssetStats,
    pub filtered_count: u64,
    pub kb_id: Uuid,
}

#[derive(serde::Serialize)]
pub struct AssetReferenceItem {
    pub content_id: Uuid,
    pub title: String,
    pub category: Option<String>,
    pub knowledge_base_id: Option<Uuid>,
    pub knowledge_base_title: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub reference_type: String,
    pub snippet: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_assets).post(upload_asset))
        .route("/:id/references", get(list_asset_references))
        .route("/:id/permissions", get(explain_asset_permissions))
        .route("/:id", get(get_asset).delete(delete_asset))
}

async fn list_assets(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<ListAssetsQuery>,
) -> Result<Json<AssetListResponse>, (StatusCode, String)> {
    let user_id = UserId(user.id);
    let kb_id = state
        .asset_manager
        .ensure_my_assets_kb(user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let items = ArticleRepository::list(
        &*state.repo,
        Some(user_id.clone()),
        Some(user_id),
        Some(kb_id),
        None,
        Some("Asset".to_string()),
        500,
        0,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut stats = AssetStats::default();
    let normalized_query = query
        .q
        .as_deref()
        .map(normalize_search_text)
        .filter(|value| !value.is_empty());
    let requested_type = normalize_asset_type(query.asset_type.as_deref());

    let filtered_assets: Vec<ContentItem> = items
        .into_iter()
        .filter_map(|item| {
            let article = match &item {
                ContentItem::Article(article) => article,
                _ => return None,
            };

            if article.category.as_deref() != Some("Asset") {
                return None;
            }

            let asset_type = asset_type_for_article(article);
            increment_asset_stats(&mut stats, asset_type.as_str());

            if let Some(expected_type) = requested_type.as_deref() {
                if asset_type != expected_type {
                    return None;
                }
            }

            if let Some(search) = normalized_query.as_deref() {
                let haystack = asset_search_blob(article);
                if !haystack.contains(search) {
                    return None;
                }
            }

            Some(item)
        })
        .collect();

    let offset = query.offset.unwrap_or(0) as usize;
    let limit = query.limit.unwrap_or(100).min(200) as usize;
    let filtered_count = filtered_assets.len() as u64;

    // Apply sort_by before pagination
    let sort_by = query
        .sort_by
        .as_deref()
        .unwrap_or("newest")
        .trim()
        .to_lowercase();
    let mut sorted_assets = filtered_assets;
    match sort_by.as_str() {
        "largest" => {
            sorted_assets.sort_by(|a, b| {
                let size_a = content_item_size_bytes(a);
                let size_b = content_item_size_bytes(b);
                size_b.cmp(&size_a)
            });
        }
        "name" => {
            sorted_assets.sort_by(|a, b| {
                let name_a = content_item_title(a).to_lowercase();
                let name_b = content_item_title(b).to_lowercase();
                name_a.cmp(&name_b)
            });
        }
        // "newest" is the default — sort by updated_at descending
        _ => {
            sorted_assets.sort_by(|a, b| {
                let date_a = content_item_updated_at(a);
                let date_b = content_item_updated_at(b);
                date_b.cmp(&date_a)
            });
        }
    }

    let paged_items = sorted_assets.into_iter().skip(offset).take(limit).collect();

    Ok(Json(AssetListResponse {
        items: paged_items,
        stats,
        filtered_count,
        kb_id,
    }))
}

async fn upload_asset(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = user.id;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let filename = field.file_name().unwrap_or("unnamed").to_string();
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let node = state
                .asset_manager
                .upload_asset(user_id, filename, content_type, &data)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

            return Ok(Json(node));
        }
    }

    Err((StatusCode::BAD_REQUEST, "No file field found".to_string()))
}

async fn list_asset_references(
    State(state): State<AppState>,
    Path(asset_id): Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<AssetReferenceItem>>, (StatusCode, String)> {
    let asset = ArticleRepository::find_by_id(&*state.repo, &asset_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Asset not found".to_string()))?;

    let asset_article = match asset {
        ContentItem::Article(article) if article.category.as_deref() == Some("Asset") => article,
        _ => return Err((StatusCode::NOT_FOUND, "Asset not found".to_string())),
    };

    if asset_article.node.author_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Unauthorized".to_string()));
    }

    let items = ArticleRepository::list(
        &*state.repo,
        Some(UserId(user.id)),
        Some(UserId(user.id)),
        None,
        None,
        None,
        1000,
        0,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let asset_marker = format!("[[asset:{}]]", asset_id);
    let embed_marker = format!("([[asset:{}]])", asset_id);

    let mut references = Vec::new();
    for item in items {
        let article = match item {
            ContentItem::Article(article) => article,
            _ => continue,
        };

        if article.node.id == asset_id || article.category.as_deref() == Some("Asset") {
            continue;
        }

        let Some((reference_type, snippet)) =
            extract_asset_reference_context(&article, &asset_marker, &embed_marker)
        else {
            continue;
        };

        let knowledge_base_title = if let Some(kb_id) = article.node.knowledge_base_id {
            KnowledgeBaseRepository::find_by_id(&*state.repo, &KnowledgeBaseId(kb_id))
                .await
                .ok()
                .flatten()
                .map(|kb| kb.title)
        } else {
            None
        };

        references.push(AssetReferenceItem {
            content_id: article.node.id,
            title: article.node.title,
            category: article.category,
            knowledge_base_id: article.node.knowledge_base_id,
            knowledge_base_title,
            updated_at: article.node.updated_at,
            reference_type: reference_type.to_string(),
            snippet,
        });
    }

    references.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(Json(references))
}

async fn explain_asset_permissions(
    State(state): State<AppState>,
    Path(asset_id): Path<Uuid>,
    Query(query): Query<AssetQuery>,
    user: AuthenticatedUser,
) -> Result<Json<crate::domain::permission_service::PermissionExplanation>, (StatusCode, String)> {
    let explanation = state
        .asset_manager
        .explain_asset_access(asset_id, query.context, user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(explanation))
}

async fn get_asset(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<AssetQuery>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = user.id;

    let (path, mime_type) = state
        .asset_manager
        .get_asset_file(id, query.context, user_id)
        .await
        .map_err(|e| (StatusCode::FORBIDDEN, e))?;

    let file = File::open(&path)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "File not found on disk".to_string()))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, mime_type),
        (header::CACHE_CONTROL, "public, max-age=31536000".to_string()),
    ];

    Ok((headers, body))
}

async fn delete_asset(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let asset = ArticleRepository::find_by_id(&*state.repo, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Asset not found".to_string()))?;

    let asset_article = match asset {
        ContentItem::Article(article) if article.category.as_deref() == Some("Asset") => article,
        _ => return Err((StatusCode::NOT_FOUND, "Asset not found".to_string())),
    };

    if asset_article.node.author_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Unauthorized".to_string()));
    }

    // We intentionally keep the blob on disk for now because uploads are content-addressable
    // and multiple asset records can point to the same file. A later GC pass can prune orphans.
    ArticleRepository::delete(&*state.repo, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

fn normalize_search_text(input: &str) -> String {
    input.trim().to_lowercase()
}

fn normalize_asset_type(asset_type: Option<&str>) -> Option<String> {
    asset_type
        .map(|value| value.trim().to_lowercase())
        .filter(|value| {
            matches!(
                value.as_str(),
                "image_asset"
                    | "pdf_asset"
                    | "file_asset"
                    | "ip_asset"
                    | "domain_asset"
                    | "credential_stub"
                    | "snippet_asset"
            )
        })
}

fn asset_payload(article: &Article) -> Option<&serde_json::Value> {
    match &article.body {
        ContentBody::Custom(payload) => Some(payload),
        _ => None,
    }
}

fn asset_type_for_article(article: &Article) -> String {
    let payload = match asset_payload(article) {
        Some(payload) => payload,
        None => return "file_asset".to_string(),
    };

    if let Some(asset_type) = payload.get("asset_type").and_then(|value| value.as_str()) {
        let normalized = asset_type.trim().to_lowercase();
        if matches!(
            normalized.as_str(),
            "image_asset"
                | "pdf_asset"
                | "file_asset"
                | "ip_asset"
                | "domain_asset"
                | "credential_stub"
                | "snippet_asset"
        ) {
            return normalized;
        }
    }

    let mime_type = payload
        .get("mime_type")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim()
        .to_lowercase();
    if mime_type.starts_with("image/") {
        return "image_asset".to_string();
    }

    let extension = payload
        .pointer("/metadata/extension")
        .and_then(|value| value.as_str())
        .or_else(|| {
            payload
                .get("original_filename")
                .and_then(|value| value.as_str())
                .and_then(|filename| filename.rsplit('.').next())
        })
        .unwrap_or("")
        .trim()
        .to_lowercase();

    if mime_type == "application/pdf" || extension == "pdf" {
        return "pdf_asset".to_string();
    }

    "file_asset".to_string()
}

fn asset_search_blob(article: &Article) -> String {
    let payload = match asset_payload(article) {
        Some(payload) => payload,
        None => return article.node.title.to_lowercase(),
    };

    [
        article.node.title.as_str(),
        payload
            .get("display_name")
            .and_then(|value| value.as_str())
            .unwrap_or(""),
        payload
            .get("original_filename")
            .and_then(|value| value.as_str())
            .unwrap_or(""),
        payload
            .get("mime_type")
            .and_then(|value| value.as_str())
            .unwrap_or(""),
        payload
            .get("hash")
            .and_then(|value| value.as_str())
            .unwrap_or(""),
        payload
            .pointer("/metadata/extension")
            .and_then(|value| value.as_str())
            .unwrap_or(""),
    ]
    .join(" ")
    .to_lowercase()
}

fn article_body_text(article: &Article) -> String {
    match &article.body {
        ContentBody::Markdown(text) => text.clone(),
        ContentBody::CodeSnippet { code, .. } => code.clone(),
        ContentBody::Custom(payload) => payload.to_string(),
        _ => String::new(),
    }
}

fn content_item_size_bytes(item: &ContentItem) -> u64 {
    match item {
        ContentItem::Article(article) => {
            asset_payload(article)
                .and_then(|p| p.get("size_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
        }
        _ => 0,
    }
}

fn content_item_title(item: &ContentItem) -> &str {
    match item {
        ContentItem::Article(article) => article.node.title.as_str(),
        _ => "",
    }
}

fn content_item_updated_at(item: &ContentItem) -> chrono::DateTime<chrono::Utc> {
    match item {
        ContentItem::Article(article) => article.node.updated_at,
        _ => chrono::Utc::now(),
    }
}

fn extract_asset_reference_context(
    article: &Article,
    asset_marker: &str,
    embed_marker: &str,
) -> Option<(&'static str, String)> {
    let body = article_body_text(article);
    if body.is_empty() {
        return None;
    }

    let (reference_type, index, needle) = if let Some(index) = body.find(embed_marker) {
        ("embed", index, embed_marker)
    } else if let Some(index) = body.find(asset_marker) {
        ("reference", index, asset_marker)
    } else {
        return None;
    };

    let chars: Vec<char> = body.chars().collect();
    let start_char = body[..index].chars().count().saturating_sub(48);
    let marker_char_len = needle.chars().count();
    let marker_start_char = body[..index].chars().count();
    let end_char = (marker_start_char + marker_char_len + 48).min(chars.len());

    let snippet = chars[start_char..end_char]
        .iter()
        .collect::<String>()
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    Some((reference_type, snippet))
}

fn increment_asset_stats(stats: &mut AssetStats, asset_type: &str) {
    stats.total += 1;
    match asset_type {
        "image_asset" => stats.images += 1,
        "pdf_asset" => stats.pdfs += 1,
        "ip_asset" => stats.ip_assets += 1,
        "domain_asset" => stats.domain_assets += 1,
        "credential_stub" => stats.credential_stubs += 1,
        "snippet_asset" => stats.snippets += 1,
        _ => stats.files += 1,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        asset_search_blob, asset_type_for_article, extract_asset_reference_context,
        increment_asset_stats, normalize_asset_type, normalize_search_text, AssetStats,
    };
    use crate::domain::models::{Article, ContentBody, ContentStatus, Node, NodeType, PermissionMode};
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    fn make_asset_article(payload: serde_json::Value) -> Article {
        Article {
            node: Node {
                id: Uuid::new_v4(),
                parent_id: None,
                author_id: Uuid::new_v4(),
                knowledge_base_id: Some(Uuid::new_v4()),
                r#type: NodeType::Article,
                title: "Asset Node".to_string(),
                permission_mode: PermissionMode::Private,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            slug: "asset".to_string(),
            status: ContentStatus::Published,
            category: Some("Asset".to_string()),
            body: ContentBody::Custom(payload),
            tags: vec![],
            author_name: None,
            author_avatar: None,
            derived_data: None,
            analysis_status: None,
            analysis_diagnostics: None,
        }
    }

    #[test]
    fn resolves_asset_type_from_payload_and_legacy_fields() {
        let typed = make_asset_article(json!({
            "asset_type": "image_asset",
            "mime_type": "image/png"
        }));
        let legacy_pdf = make_asset_article(json!({
            "original_filename": "paper.pdf",
            "mime_type": "application/octet-stream"
        }));
        let generic = make_asset_article(json!({
            "original_filename": "archive.zip",
            "mime_type": "application/zip"
        }));

        assert_eq!(asset_type_for_article(&typed), "image_asset");
        assert_eq!(asset_type_for_article(&legacy_pdf), "pdf_asset");
        assert_eq!(asset_type_for_article(&generic), "file_asset");
    }

    #[test]
    fn builds_search_blob_from_asset_metadata() {
        let article = make_asset_article(json!({
            "display_name": "diagram.png",
            "original_filename": "diagram.png",
            "mime_type": "image/png",
            "hash": "abc123",
            "metadata": {
                "extension": "png"
            }
        }));

        let blob = asset_search_blob(&article);
        assert!(blob.contains("diagram.png"));
        assert!(blob.contains("image/png"));
        assert!(blob.contains("abc123"));
    }

    #[test]
    fn normalizes_asset_queries_and_stats() {
        assert_eq!(normalize_search_text("  Diagram  "), "diagram");
        assert_eq!(normalize_asset_type(Some(" PDF_ASSET ")).as_deref(), Some("pdf_asset"));
        assert_eq!(normalize_asset_type(Some("credential_stub")).as_deref(), Some("credential_stub"));
        assert_eq!(normalize_asset_type(Some("ip_asset")).as_deref(), Some("ip_asset"));
        assert_eq!(normalize_asset_type(Some("domain_asset")).as_deref(), Some("domain_asset"));
        assert_eq!(normalize_asset_type(Some("snippet_asset")).as_deref(), Some("snippet_asset"));
        assert_eq!(normalize_asset_type(Some("unknown_type")), None);

        let mut stats = AssetStats::default();
        increment_asset_stats(&mut stats, "image_asset");
        increment_asset_stats(&mut stats, "pdf_asset");
        increment_asset_stats(&mut stats, "file_asset");
        increment_asset_stats(&mut stats, "ip_asset");
        increment_asset_stats(&mut stats, "domain_asset");
        increment_asset_stats(&mut stats, "credential_stub");
        increment_asset_stats(&mut stats, "snippet_asset");

        assert_eq!(stats.total, 7);
        assert_eq!(stats.images, 1);
        assert_eq!(stats.pdfs, 1);
        assert_eq!(stats.files, 1);
        assert_eq!(stats.ip_assets, 1);
        assert_eq!(stats.domain_assets, 1);
        assert_eq!(stats.credential_stubs, 1);
        assert_eq!(stats.snippets, 1);
    }

    #[test]
    fn extracts_reference_context_from_markdown_assets() {
        let article = make_asset_article(json!({}));
        let content = Article {
            body: ContentBody::Markdown(format!(
                "Before text ![diagram]([[asset:{}]]) and also trailing context",
                article.node.id
            )),
            category: Some("Note".to_string()),
            ..article.clone()
        };

        let result = extract_asset_reference_context(
            &content,
            &format!("[[asset:{}]]", article.node.id),
            &format!("([[asset:{}]])", article.node.id),
        )
        .expect("reference should be found");

        assert_eq!(result.0, "embed");
        assert!(result.1.contains("[[asset:"));
        assert!(result.1.contains("Before text"));
    }
}
