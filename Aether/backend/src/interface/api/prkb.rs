use crate::domain::prkb::models::{Author, Collection, Feed, Paper, Signals};
use crate::domain::prkb::ports::PrkbRepository;
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ===== DTOs =====

#[derive(Deserialize)]
pub struct CreateFeedRequest {
    pub name: String,
    pub url: String,
    pub feed_type: String,
}

#[derive(Deserialize)]
pub struct UpdateFeedRequest {
    pub enabled: Option<bool>,
}

#[derive(Deserialize)]
pub struct InboxQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub unread_only: Option<bool>,
    pub publication: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateInboxItemRequest {
    pub state: Option<String>,
    pub is_read: Option<bool>,
    pub priority: Option<i32>,
    pub note: Option<String>,
}

#[derive(Deserialize)]
pub struct SavePaperRequest {
    pub inbox_item_id: Option<Uuid>,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub url: String,
    pub pdf_url: Option<String>,
    pub source: String,
    pub publish_date: chrono::DateTime<chrono::Utc>,
    pub arxiv_id: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct FetchFeedsRequest {
    pub feed_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct FetchStats {
    pub total_count: usize,
    pub details: Vec<FeedFetchResult>,
}

#[derive(Serialize)]
pub struct FeedFetchResult {
    pub feed_name: String,
    pub count: usize,
    pub status: String,
}

#[derive(Deserialize)]
pub struct ListPapersQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub venue_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub is_read: Option<bool>,
    // PRKB-04: Search / DSL
    pub q: Option<String>,
    pub state: Option<String>,
    pub tag: Option<String>,
    pub year: Option<i32>,
    pub has_pdf: Option<bool>,
    pub pdf_status: Option<String>,
    pub collection_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdatePaperRequest {
    pub is_read: Option<bool>,
    pub state: Option<String>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
    pub pdf_status: Option<String>,
    pub pdf_local_path: Option<String>,
}

// PRKB-05: Collections
#[derive(Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub collection_type: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateCollectionRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct CollectionPaperRequest {
    pub paper_id: Uuid,
}

// PRKB-07: Signals
#[derive(Deserialize)]
pub struct UpdateSignalsRequest {
    pub citation_count: Option<i32>,
    pub github_stars: Option<i32>,
    pub feed_freshness: Option<String>,
    pub venue_tier: Option<String>,
    pub author_recurrence: Option<i32>,
    pub custom_importance: Option<i32>,
}

// PRKB-08: Portability
#[derive(Deserialize)]
pub struct ExportRequest {
    pub format: String,
    pub collection_id: Option<Uuid>,
    pub paper_ids: Option<Vec<Uuid>>,
}

#[derive(Deserialize)]
pub struct ImportBibtexRequest {
    pub bibtex: String,
    pub merge_tags: Option<bool>,
    pub merge_notes: Option<bool>,
}

// ===== HANDLER HELPERS =====

fn err_response(e: impl std::fmt::Display) -> axum::http::Response<axum::body::Body> {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": e.to_string()})),
    )
        .into_response()
}

fn ok_json<T: Serialize>(data: T) -> axum::http::Response<axum::body::Body> {
    (StatusCode::OK, Json(data)).into_response()
}

// ===== PRKB-01: FEED HANDLERS =====

pub async fn list_feeds(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    match state.repo.list_feeds().await {
        Ok(feeds) => ok_json(feeds),
        Err(e) => err_response(e),
    }
}

pub async fn create_feed(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<CreateFeedRequest>,
) -> impl IntoResponse {
    let feed = Feed {
        id: Uuid::new_v4(),
        name: payload.name,
        url: payload.url,
        feed_type: payload.feed_type,
        enabled: true,
        last_fetched_at: None,
        created_at: chrono::Utc::now(),
        health_status: "unknown".to_string(),
        total_fetched: 0,
        parse_errors: 0,
        last_error: None,
    };

    match state.repo.create_feed(feed).await {
        Ok(id) => ok_json(serde_json::json!({"id": id})),
        Err(e) => err_response(e),
    }
}

pub async fn delete_feed(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.delete_feed(id).await {
        Ok(_) => ok_json(serde_json::json!({"status": "deleted"})),
        Err(e) => err_response(e),
    }
}

pub async fn update_feed(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateFeedRequest>,
) -> impl IntoResponse {
    if let Some(enabled) = payload.enabled {
        if let Err(e) = state.repo.update_feed_enabled(id, enabled).await {
            return err_response(e);
        }
    }
    ok_json(serde_json::json!({"status": "updated"}))
}

pub async fn test_feed_parser(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let feed = match state.repo.get_feed(id).await {
        Ok(Some(f)) => f,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Feed not found"})),
            )
                .into_response()
        }
        Err(e) => return err_response(e),
    };

    let result = if feed.feed_type == "arxiv" {
        state.arxiv_service.fetch_recent_by_category(&feed.url, 5).await
    } else if feed.feed_type == "rss" {
        state.rss_service.fetch_feed(&feed.url).await
    } else {
        return ok_json(serde_json::json!({"status": "error", "message": "Unknown feed type"}));
    };

    match result {
        Ok(items) => ok_json(serde_json::json!({
            "status": "ok",
            "sample_count": items.len(),
            "sample_titles": items.iter().take(3).map(|i| i.title.clone()).collect::<Vec<_>>()
        })),
        Err(e) => ok_json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}

// ===== PRKB-02: INBOX HANDLERS =====

pub async fn get_inbox(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(q): Query<InboxQuery>,
) -> impl IntoResponse {
    let unread_only = q.unread_only.unwrap_or(false);
    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);
    let publication = q.publication;

    let items_result = state
        .repo
        .get_inbox(limit, offset, unread_only, publication.clone())
        .await;
    let count_result = state.repo.count_inbox(unread_only, publication).await;

    match (items_result, count_result) {
        (Ok(items), Ok(total)) => ok_json(serde_json::json!({
            "items": items,
            "total": total,
            "limit": limit,
            "offset": offset
        })),
        (Err(e), _) | (_, Err(e)) => err_response(e),
    }
}

pub async fn update_inbox_item(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateInboxItemRequest>,
) -> impl IntoResponse {
    if let Some(s) = payload.state {
        if let Err(e) = state.repo.update_inbox_state(id, s).await {
            return err_response(e);
        }
    }
    if let Some(read) = payload.is_read {
        if read {
            if let Err(e) = state.repo.markup_inbox_item_read(id).await {
                return err_response(e);
            }
        }
    }
    if payload.priority.is_some() {
        if let Err(e) = state.repo.update_inbox_priority(id, payload.priority).await {
            return err_response(e);
        }
    }
    if payload.note.is_some() {
        if let Err(e) = state.repo.update_inbox_note(id, payload.note).await {
            return err_response(e);
        }
    }
    ok_json(serde_json::json!({"status": "updated"}))
}

pub async fn get_publications(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    match state.repo.get_unique_publications().await {
        Ok(pubs) => ok_json(pubs),
        Err(e) => err_response(e),
    }
}

// ===== FETCH HANDLER =====

pub async fn fetch_feeds(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<FetchFeedsRequest>,
) -> impl IntoResponse {
    let all_feeds = match state.repo.list_feeds().await {
        Ok(f) => f,
        Err(e) => return err_response(e),
    };

    let feeds_to_fetch: Vec<Feed> = if let Some(target_id) = payload.feed_id {
        all_feeds
            .into_iter()
            .filter(|f| f.id == target_id && f.enabled)
            .collect()
    } else {
        all_feeds.into_iter().filter(|f| f.enabled).collect()
    };

    let mut total_count = 0;
    let mut details = Vec::new();

    for feed in feeds_to_fetch {
        tracing::info!("Fetching feed: {}", feed.name);

        let items_result = if feed.feed_type == "arxiv" {
            state
                .arxiv_service
                .fetch_recent_by_category(&feed.url, 100)
                .await
        } else if feed.feed_type == "rss" {
            state.rss_service.fetch_feed(&feed.url).await
        } else {
            details.push(FeedFetchResult {
                feed_name: feed.name.clone(),
                count: 0,
                status: "skipped_unknown_type".to_string(),
            });
            continue;
        };

        match items_result {
            Ok(mut items) => {
                for item in &mut items {
                    item.feed_id = feed.id;
                }

                let new_count = items.len();
                if let Err(e) = state.repo.save_inbox_items(items).await {
                    tracing::error!("Failed to save items for feed {}: {}", feed.name, e);
                    let _ = state
                        .repo
                        .update_feed_health(
                            feed.id,
                            "error".to_string(),
                            Some(format!("Save error: {}", e)),
                        )
                        .await;
                    details.push(FeedFetchResult {
                        feed_name: feed.name.clone(),
                        count: 0,
                        status: format!("save_error: {}", e),
                    });
                } else {
                    total_count += new_count;
                    let _ = state
                        .repo
                        .update_feed_last_fetched(feed.id, chrono::Utc::now())
                        .await;
                    let _ = state
                        .repo
                        .update_feed_health(feed.id, "healthy".to_string(), None)
                        .await;
                    let _ = state
                        .repo
                        .increment_feed_stats(feed.id, new_count as i64, 0)
                        .await;

                    details.push(FeedFetchResult {
                        feed_name: feed.name.clone(),
                        count: new_count,
                        status: "ok".to_string(),
                    });
                }
            }
            Err(e) => {
                tracing::error!("Failed to fetch feed {}: {}", feed.name, e);
                let _ = state
                    .repo
                    .update_feed_health(
                        feed.id,
                        "error".to_string(),
                        Some(format!("Fetch error: {}", e)),
                    )
                    .await;
                let _ = state
                    .repo
                    .increment_feed_stats(feed.id, 0, 1)
                    .await;
                details.push(FeedFetchResult {
                    feed_name: feed.name.clone(),
                    count: 0,
                    status: format!("fetch_error: {}", e),
                });
            }
        }
    }

    ok_json(FetchStats {
        total_count,
        details,
    })
}

// ===== PRKB-03: PAPER HANDLERS =====

pub async fn save_paper(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<SavePaperRequest>,
) -> impl IntoResponse {
    let authors: Vec<Author> = payload
        .authors
        .into_iter()
        .map(|name| Author {
            id: Uuid::new_v4(),
            name,
            canonical_name: None,
            profile_url: None,
        })
        .collect();

    let paper = Paper {
        id: Uuid::new_v4(),
        title: payload.title,
        authors,
        abstract_text: payload.abstract_text,
        url: payload.url,
        pdf_url: payload.pdf_url,
        pdf_local_path: None,
        publish_date: payload.publish_date,
        source: payload.source,
        saved_at: chrono::Utc::now(),
        is_read: false,
        state: "Inbox".to_string(),
        tags: payload.tags,
        arxiv_id: payload.arxiv_id,
        venue: None,
        signals: None,
        metadata: None,
        pdf_status: "not_attached".to_string(),
        notes: None,
    };

    match state.repo.save_paper(paper).await {
        Ok(id) => {
            // Mark inbox item as saved
            if let Some(inbox_id) = payload.inbox_item_id {
                let _ = state
                    .repo
                    .update_inbox_state(inbox_id, "saved".to_string())
                    .await;
            }
            ok_json(serde_json::json!({"id": id}))
        }
        Err(e) => err_response(e),
    }
}

pub async fn list_papers(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(q): Query<ListPapersQuery>,
) -> impl IntoResponse {
    // PRKB-04: Parse DSL from q parameter
    let mut filter = crate::domain::prkb::models::PaperFilter {
        venue_id: q.venue_id,
        author_id: q.author_id,
        is_read: q.is_read,
        state: q.state,
        tag: q.tag,
        year: q.year,
        has_pdf: q.has_pdf,
        pdf_status: q.pdf_status,
        collection_id: q.collection_id,
        ..Default::default()
    };

    // Parse DSL query: author:X venue:Y year:2024 state:read tag:important freetext
    if let Some(raw_query) = &q.q {
        let mut free_text = Vec::new();
        for token in raw_query.split_whitespace() {
            if let Some(val) = token.strip_prefix("author:") {
                // For DSL: would need author name -> id lookup, simplified
                filter.query = Some(val.to_string());
            } else if let Some(val) = token.strip_prefix("venue:") {
                filter.query = Some(val.to_string());
            } else if let Some(val) = token.strip_prefix("year:") {
                if let Ok(y) = val.parse::<i32>() {
                    filter.year = Some(y);
                }
            } else if let Some(val) = token.strip_prefix("state:") {
                filter.state = Some(val.to_string());
            } else if let Some(val) = token.strip_prefix("tag:") {
                filter.tag = Some(val.to_string());
            } else {
                free_text.push(token);
            }
        }
        if !free_text.is_empty() {
            filter.query = Some(free_text.join(" "));
        }
    }

    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);

    match state.repo.list_papers(filter, limit, offset).await {
        Ok(papers) => ok_json(papers),
        Err(e) => err_response(e),
    }
}

pub async fn get_paper(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.get_paper(id).await {
        Ok(Some(paper)) => ok_json(paper),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Paper not found"})),
        )
            .into_response(),
        Err(e) => err_response(e),
    }
}

pub async fn update_paper(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePaperRequest>,
) -> impl IntoResponse {
    if let Some(is_read) = payload.is_read {
        if let Err(e) = state.repo.update_paper_read_status(id, is_read).await {
            return err_response(e);
        }
    }
    if let Some(s) = payload.state {
        if let Err(e) = state.repo.update_paper_state(id, s).await {
            return err_response(e);
        }
    }
    if let Some(tags) = payload.tags {
        if let Err(e) = state.repo.update_paper_tags(id, tags).await {
            return err_response(e);
        }
    }
    if let Some(notes) = payload.notes {
        if let Err(e) = state.repo.update_paper_notes(id, Some(notes)).await {
            return err_response(e);
        }
    }
    if let Some(pdf_status) = payload.pdf_status {
        if let Err(e) = state
            .repo
            .update_paper_pdf_status(id, pdf_status, payload.pdf_local_path)
            .await
        {
            return err_response(e);
        }
    }
    ok_json(serde_json::json!({"status": "updated"}))
}

pub async fn list_venues(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    match state.repo.list_venues().await {
        Ok(venues) => ok_json(venues),
        Err(e) => err_response(e),
    }
}

// ===== PRKB-05: COLLECTION HANDLERS =====

pub async fn list_collections(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    match state.repo.list_collections().await {
        Ok(collections) => ok_json(collections),
        Err(e) => err_response(e),
    }
}

pub async fn create_collection(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<CreateCollectionRequest>,
) -> impl IntoResponse {
    let now = chrono::Utc::now();
    let collection = Collection {
        id: Uuid::new_v4(),
        name: payload.name,
        collection_type: payload.collection_type,
        description: payload.description,
        paper_count: 0,
        created_at: now,
        updated_at: now,
    };
    match state.repo.create_collection(collection).await {
        Ok(id) => ok_json(serde_json::json!({"id": id})),
        Err(e) => err_response(e),
    }
}

pub async fn update_collection(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCollectionRequest>,
) -> impl IntoResponse {
    match state
        .repo
        .update_collection(id, payload.name, payload.description)
        .await
    {
        Ok(_) => ok_json(serde_json::json!({"status": "updated"})),
        Err(e) => err_response(e),
    }
}

pub async fn delete_collection(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.delete_collection(id).await {
        Ok(_) => ok_json(serde_json::json!({"status": "deleted"})),
        Err(e) => err_response(e),
    }
}

pub async fn add_to_collection(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<CollectionPaperRequest>,
) -> impl IntoResponse {
    match state
        .repo
        .add_paper_to_collection(id, payload.paper_id)
        .await
    {
        Ok(_) => ok_json(serde_json::json!({"status": "added"})),
        Err(e) => err_response(e),
    }
}

pub async fn remove_from_collection(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path((id, paper_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match state
        .repo
        .remove_paper_from_collection(id, paper_id)
        .await
    {
        Ok(_) => ok_json(serde_json::json!({"status": "removed"})),
        Err(e) => err_response(e),
    }
}

pub async fn list_collection_papers(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Query(q): Query<ListPapersQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);
    match state
        .repo
        .list_collection_papers(id, limit, offset)
        .await
    {
        Ok(papers) => ok_json(papers),
        Err(e) => err_response(e),
    }
}

// ===== PRKB-07: SIGNAL HANDLERS =====

pub async fn update_signals(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSignalsRequest>,
) -> impl IntoResponse {
    // Get existing signals first
    let paper = match state.repo.get_paper(id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Paper not found"})),
            )
                .into_response()
        }
        Err(e) => return err_response(e),
    };

    let existing = paper.signals.unwrap_or(Signals {
        citation_count: 0,
        github_stars: 0,
        sota_rank: None,
        last_updated: chrono::Utc::now(),
        feed_freshness: None,
        venue_tier: None,
        author_recurrence: None,
        custom_importance: None,
    });

    let signals = Signals {
        citation_count: payload.citation_count.unwrap_or(existing.citation_count),
        github_stars: payload.github_stars.unwrap_or(existing.github_stars),
        sota_rank: existing.sota_rank,
        last_updated: chrono::Utc::now(),
        feed_freshness: payload.feed_freshness.or(existing.feed_freshness),
        venue_tier: payload.venue_tier.or(existing.venue_tier),
        author_recurrence: payload.author_recurrence.or(existing.author_recurrence),
        custom_importance: payload.custom_importance.or(existing.custom_importance),
    };

    match state.repo.update_paper_signals(id, signals).await {
        Ok(_) => ok_json(serde_json::json!({"status": "updated"})),
        Err(e) => err_response(e),
    }
}

// ===== PRKB-08: PORTABILITY HANDLERS =====

pub async fn export_papers(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<ExportRequest>,
) -> impl IntoResponse {
    // Gather papers
    let papers = if let Some(ids) = payload.paper_ids {
        let mut result = Vec::new();
        for id in ids {
            if let Ok(Some(p)) = state.repo.get_paper(id).await {
                result.push(p);
            }
        }
        result
    } else if let Some(cid) = payload.collection_id {
        state
            .repo
            .list_collection_papers(cid, 1000, 0)
            .await
            .unwrap_or_default()
    } else {
        let filter = crate::domain::prkb::models::PaperFilter::default();
        state
            .repo
            .list_papers(filter, 1000, 0)
            .await
            .unwrap_or_default()
    };

    match payload.format.as_str() {
        "bibtex" => {
            let bibtex = papers_to_bibtex(&papers);
            (StatusCode::OK, [("content-type", "text/plain")], bibtex).into_response()
        }
        "json" => ok_json(&papers),
        "markdown" => {
            let md = papers_to_markdown_digest(&papers);
            (StatusCode::OK, [("content-type", "text/markdown")], md).into_response()
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Unsupported format. Use: bibtex, json, markdown"})),
        )
            .into_response(),
    }
}

pub async fn import_bibtex(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<ImportBibtexRequest>,
) -> impl IntoResponse {
    let merge_tags = payload.merge_tags.unwrap_or(true);
    let merge_notes = payload.merge_notes.unwrap_or(true);
    let entries = parse_bibtex_entries(&payload.bibtex);

    let mut imported = 0;
    let mut duplicates = 0;
    let mut errors = 0;
    let mut details = Vec::new();

    for entry in entries {
        // Dedupe by DOI -> external_id -> title
        let existing = if let Some(doi) = &entry.doi {
            state.repo.find_paper_by_doi(doi).await.ok().flatten()
        } else if let Some(eid) = &entry.arxiv_id {
            state
                .repo
                .find_paper_by_external_id(eid)
                .await
                .ok()
                .flatten()
        } else {
            state
                .repo
                .find_paper_by_title(&entry.title)
                .await
                .ok()
                .flatten()
        };

        if let Some(existing_paper) = existing {
            duplicates += 1;
            // Merge tags/notes if requested
            if merge_tags && !entry.tags.is_empty() {
                let mut merged_tags = existing_paper.tags.clone();
                for t in &entry.tags {
                    if !merged_tags.contains(t) {
                        merged_tags.push(t.clone());
                    }
                }
                let _ = state
                    .repo
                    .update_paper_tags(existing_paper.id, merged_tags)
                    .await;
            }
            if merge_notes {
                if let Some(note) = &entry.notes {
                    let merged = match &existing_paper.notes {
                        Some(existing_notes) => {
                            format!("{}\n---\n{}", existing_notes, note)
                        }
                        None => note.clone(),
                    };
                    let _ = state
                        .repo
                        .update_paper_notes(existing_paper.id, Some(merged))
                        .await;
                }
            }
            details.push(format!("Duplicate (merged): {}", entry.title));
        } else {
            // Create new
            let authors: Vec<Author> = entry
                .authors
                .iter()
                .map(|name| Author {
                    id: Uuid::new_v4(),
                    name: name.clone(),
                    canonical_name: None,
                    profile_url: None,
                })
                .collect();

            let metadata = crate::domain::prkb::models::PaperMetadata {
                track: None,
                series: None,
                bibtex: Some(crate::domain::prkb::models::BibTexInfo {
                    publisher: entry.publisher.clone(),
                    editor: None,
                    pages: entry.pages.clone(),
                    doi: entry.doi.clone(),
                    isbn: None,
                }),
                subjects: vec![],
                keywords: entry.tags.clone(),
            };

            let paper = Paper {
                id: Uuid::new_v4(),
                title: entry.title.clone(),
                authors,
                abstract_text: entry.abstract_text.clone().unwrap_or_default(),
                url: entry.url.clone().unwrap_or_default(),
                pdf_url: None,
                pdf_local_path: None,
                publish_date: entry.year.map_or(chrono::Utc::now(), |y| {
                    chrono::NaiveDate::from_ymd_opt(y, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_utc()
                }),
                source: "bibtex_import".to_string(),
                saved_at: chrono::Utc::now(),
                is_read: false,
                state: "Inbox".to_string(),
                tags: entry.tags.clone(),
                arxiv_id: entry.arxiv_id.clone(),
                venue: entry.venue.as_ref().map(|v| crate::domain::prkb::models::Venue {
                    id: Uuid::new_v4(),
                    name: v.clone(),
                    tier: None,
                }),
                signals: None,
                metadata: Some(metadata),
                pdf_status: "not_attached".to_string(),
                notes: entry.notes.clone(),
            };

            match state.repo.save_paper(paper).await {
                Ok(_) => {
                    imported += 1;
                    details.push(format!("Imported: {}", entry.title));
                }
                Err(e) => {
                    errors += 1;
                    details.push(format!("Error importing {}: {}", entry.title, e));
                }
            }
        }
    }

    ok_json(serde_json::json!({
        "imported": imported,
        "duplicates": duplicates,
        "errors": errors,
        "details": details
    }))
}

// ===== BIBTEX HELPERS =====

struct BibtexEntry {
    title: String,
    authors: Vec<String>,
    year: Option<i32>,
    venue: Option<String>,
    doi: Option<String>,
    url: Option<String>,
    arxiv_id: Option<String>,
    abstract_text: Option<String>,
    publisher: Option<String>,
    pages: Option<String>,
    tags: Vec<String>,
    notes: Option<String>,
}

fn parse_bibtex_entries(bibtex: &str) -> Vec<BibtexEntry> {
    let mut entries = Vec::new();
    let mut current_entry: Option<std::collections::HashMap<String, String>> = None;

    for line in bibtex.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('@') {
            if let Some(fields) = current_entry.take() {
                if let Some(entry) = bibtex_fields_to_entry(fields) {
                    entries.push(entry);
                }
            }
            current_entry = Some(std::collections::HashMap::new());
        } else if let Some(ref mut fields) = current_entry {
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_lowercase();
                let val = trimmed[eq_pos + 1..]
                    .trim()
                    .trim_matches(|c| c == '{' || c == '}' || c == '"' || c == ',')
                    .to_string();
                fields.insert(key, val);
            }
        }
    }

    if let Some(fields) = current_entry {
        if let Some(entry) = bibtex_fields_to_entry(fields) {
            entries.push(entry);
        }
    }

    entries
}

fn bibtex_fields_to_entry(
    fields: std::collections::HashMap<String, String>,
) -> Option<BibtexEntry> {
    let title = fields.get("title")?.clone();
    let authors = fields
        .get("author")
        .map(|a| a.split(" and ").map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    Some(BibtexEntry {
        title,
        authors,
        year: fields.get("year").and_then(|y| y.parse().ok()),
        venue: fields
            .get("booktitle")
            .or(fields.get("journal"))
            .cloned(),
        doi: fields.get("doi").cloned(),
        url: fields.get("url").cloned(),
        arxiv_id: fields.get("eprint").or(fields.get("arxivid")).cloned(),
        abstract_text: fields.get("abstract").cloned(),
        publisher: fields.get("publisher").cloned(),
        pages: fields.get("pages").cloned(),
        tags: fields
            .get("keywords")
            .map(|k| k.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        notes: fields.get("note").or(fields.get("annote")).cloned(),
    })
}

fn papers_to_bibtex(papers: &[Paper]) -> String {
    let mut output = String::new();
    for paper in papers {
        let key = paper
            .arxiv_id
            .clone()
            .unwrap_or_else(|| paper.id.to_string().chars().take(8).collect());

        let author_str = paper
            .authors
            .iter()
            .map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join(" and ");

        let year = paper.publish_date.format("%Y").to_string();

        output.push_str(&format!("@article{{{},\n", key));
        output.push_str(&format!("  title = {{{}}},\n", paper.title));
        output.push_str(&format!("  author = {{{}}},\n", author_str));
        output.push_str(&format!("  year = {{{}}},\n", year));

        if let Some(venue) = &paper.venue {
            output.push_str(&format!("  journal = {{{}}},\n", venue.name));
        }
        if !paper.url.is_empty() {
            output.push_str(&format!("  url = {{{}}},\n", paper.url));
        }
        if let Some(meta) = &paper.metadata {
            if let Some(bib) = &meta.bibtex {
                if let Some(doi) = &bib.doi {
                    output.push_str(&format!("  doi = {{{}}},\n", doi));
                }
                if let Some(pages) = &bib.pages {
                    output.push_str(&format!("  pages = {{{}}},\n", pages));
                }
            }
        }
        if !paper.abstract_text.is_empty() {
            output.push_str(&format!(
                "  abstract = {{{}}},\n",
                paper.abstract_text.chars().take(500).collect::<String>()
            ));
        }
        if !paper.tags.is_empty() {
            output.push_str(&format!("  keywords = {{{}}},\n", paper.tags.join(", ")));
        }
        output.push_str("}\n\n");
    }
    output
}

fn papers_to_markdown_digest(papers: &[Paper]) -> String {
    let mut output = String::from("# Research Library Digest\n\n");
    output.push_str(&format!("*Exported: {}*\n\n", chrono::Utc::now().format("%Y-%m-%d")));
    output.push_str(&format!("**Total papers: {}**\n\n---\n\n", papers.len()));

    for (i, paper) in papers.iter().enumerate() {
        output.push_str(&format!("## {}. {}\n\n", i + 1, paper.title));

        let authors = paper
            .authors
            .iter()
            .map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!("**Authors:** {}\n\n", authors));

        if let Some(venue) = &paper.venue {
            output.push_str(&format!("**Venue:** {}\n\n", venue.name));
        }

        output.push_str(&format!(
            "**Year:** {}\n\n",
            paper.publish_date.format("%Y")
        ));

        if !paper.tags.is_empty() {
            output.push_str(&format!("**Tags:** {}\n\n", paper.tags.join(", ")));
        }

        if !paper.abstract_text.is_empty() {
            output.push_str(&format!(
                "> {}\n\n",
                paper.abstract_text.chars().take(300).collect::<String>()
            ));
        }

        if !paper.url.is_empty() {
            output.push_str(&format!("[Link]({})\n\n", paper.url));
        }

        output.push_str("---\n\n");
    }
    output
}

// ===== ROUTER =====

pub fn router() -> Router<AppState> {
    Router::new()
        // PRKB-01: Feeds
        .route("/api/prkb/feeds", get(list_feeds).post(create_feed))
        .route("/api/prkb/feeds/:id", delete(delete_feed).patch(update_feed))
        .route("/api/prkb/feeds/:id/test", post(test_feed_parser))
        // PRKB-02: Inbox
        .route("/api/prkb/inbox", get(get_inbox))
        .route("/api/prkb/inbox/:id", patch(update_inbox_item))
        .route("/api/prkb/publications", get(get_publications))
        // Fetch
        .route("/api/prkb/fetch", post(fetch_feeds))
        // PRKB-03/04/06: Papers
        .route("/api/prkb/papers", get(list_papers).post(save_paper))
        .route(
            "/api/prkb/papers/:id",
            get(get_paper).patch(update_paper),
        )
        // PRKB-07: Signals
        .route("/api/prkb/papers/:id/signals", patch(update_signals))
        // Venues
        .route("/api/prkb/venues", get(list_venues))
        // PRKB-05: Collections
        .route(
            "/api/prkb/collections",
            get(list_collections).post(create_collection),
        )
        .route(
            "/api/prkb/collections/:id",
            patch(update_collection).delete(delete_collection),
        )
        .route(
            "/api/prkb/collections/:id/papers",
            get(list_collection_papers).post(add_to_collection),
        )
        .route(
            "/api/prkb/collections/:id/papers/:paper_id",
            delete(remove_from_collection),
        )
        // PRKB-08: Portability
        .route("/api/prkb/export", post(export_papers))
        .route("/api/prkb/import/bibtex", post(import_bibtex))
}
