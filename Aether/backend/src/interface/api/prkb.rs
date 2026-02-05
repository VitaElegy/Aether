use axum::{
    extract::{Path, State, Query},
    routing::{get, post, delete, patch},
    Json, Router, response::IntoResponse, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::interface::state::AppState;
use crate::interface::api::auth::AuthenticatedUser;
use crate::domain::prkb::models::{Feed, Paper, Author};
use crate::domain::prkb::ports::PrkbRepository;

// --- DTOs ---
#[derive(Deserialize)]
pub struct CreateFeedRequest {
    pub name: String,
    pub url: String, // or category for arxiv
    pub feed_type: String, // 'arxiv', 'rss'
}

#[derive(Deserialize)]
pub struct InboxQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub unread_only: Option<bool>,
    pub publication: Option<String>,
}

#[derive(Deserialize)]
pub struct SavePaperRequest {
    pub inbox_item_id: Option<Uuid>, // Optional: if coming from inbox
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
    pub status: String, // "ok", "error"
}

// --- HANDLERS ---

pub async fn list_feeds(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    match state.repo.list_feeds().await {
        Ok(feeds) => (StatusCode::OK, Json(feeds)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
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
        last_fetched_at: None,
        created_at: chrono::Utc::now(),
    };
    
    match state.repo.create_feed(feed).await {
        Ok(id) => (StatusCode::OK, Json(serde_json::json!({"id": id}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn delete_feed(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.delete_feed(id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "deleted"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_inbox(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(q): Query<InboxQuery>,
) -> impl IntoResponse {
    let unread_only = q.unread_only.unwrap_or(false);
    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);
    let publication = q.publication;

    let items_result = state.repo.get_inbox(limit, offset, unread_only, publication.clone()).await;
    let count_result = state.repo.count_inbox(unread_only, publication).await;

    match (items_result, count_result) {
        (Ok(items), Ok(total)) => (StatusCode::OK, Json(serde_json::json!({
            "items": items,
            "total": total,
            "limit": limit,
            "offset": offset
        }))).into_response(),
        (Err(e), _) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        (_, Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn update_inbox_item(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePaperRequest>, // Reuse request struct or create new
) -> impl IntoResponse {
    // We reuse UpdatePaperRequest for simplicity if fields match ("state", "is_read")
    // Note: Request struct has Option<String> state, Option<bool> is_read.
    
    if let Some(s) = payload.state {
        if let Err(e) = state.repo.update_inbox_state(id, s).await {
              return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response();
        }
    }
    
    if let Some(read) = payload.is_read {
        if read {
             if let Err(e) = state.repo.markup_inbox_item_read(id).await {
                  return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response();
             }
        }
    }

    (StatusCode::OK, Json(serde_json::json!({"status": "updated"}))).into_response()
}


pub async fn get_publications(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    match state.repo.get_unique_publications().await {
        Ok(pubs) => (StatusCode::OK, Json(pubs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn fetch_feeds(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<FetchFeedsRequest>,
) -> impl IntoResponse {
    // 1. List feeds
    let all_feeds = match state.repo.list_feeds().await {
        Ok(f) => f,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    };

    // Filter if feed_id provided
    let feeds_to_fetch: Vec<Feed> = if let Some(target_id) = payload.feed_id {
        all_feeds.into_iter().filter(|f| f.id == target_id).collect()
    } else {
        all_feeds
    };

    let mut total_count = 0;
    let mut details = Vec::new();
    
    // 2. Iterate and fetch
    for feed in feeds_to_fetch {
        tracing::info!("Fetching feed: {}", feed.name);
        
        let items_result = if feed.feed_type == "arxiv" {
            state.arxiv_service.fetch_recent_by_category(&feed.url, 100).await
        } else if feed.feed_type == "rss" {
            state.rss_service.fetch_feed(&feed.url).await
        } else {
            details.push(FeedFetchResult { 
                feed_name: feed.name.clone(), 
                count: 0, 
                status: "skipped_unknown_type".to_string() 
            });
            continue; 
        };

        match items_result {
            Ok(mut items) => {
                 // Update feed_id
                for item in &mut items {
                    item.feed_id = feed.id;
                }
                
                if let Err(e) = state.repo.save_inbox_items(items.clone()).await {
                     tracing::error!("Failed to save items for feed {}: {}", feed.name, e);
                     details.push(FeedFetchResult { 
                        feed_name: feed.name.clone(), 
                        count: 0, 
                        status: format!("save_error: {}", e)
                    });
                } else {
                    let new_count = items.len();
                    total_count += new_count;
                    tracing::info!("Saved {} items for feed {}", new_count, feed.name);
                    let _ = state.repo.update_feed_last_fetched(feed.id, chrono::Utc::now()).await;
                    
                    details.push(FeedFetchResult { 
                        feed_name: feed.name.clone(), 
                        count: new_count, 
                        status: "ok".to_string()
                    });
                }
            },
            Err(e) => {
                tracing::error!("Failed to fetch feed {}: {}", feed.name, e);
                details.push(FeedFetchResult { 
                    feed_name: feed.name.clone(), 
                    count: 0, 
                    status: format!("fetch_error: {}", e)
                });
            }
        }
    }

    (StatusCode::OK, Json(FetchStats { total_count, details })).into_response()
}

pub async fn save_paper(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<SavePaperRequest>,
) -> impl IntoResponse {

    let authors: Vec<Author> = payload.authors.into_iter().map(|name| Author {
        id: Uuid::new_v4(),
        name,
        canonical_name: None,
        profile_url: None,
    }).collect();

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

    };
    
    match state.repo.save_paper(paper).await {
        Ok(id) => {
            // Markup inbox item as saved if provided
            if let Some(_inbox_id) = payload.inbox_item_id {
                 // We don't have a direct method to mark "saved" in repo yet?
                 // Wait, PrkbRepository has generic inbox update? No. 
                 // We added 'is_saved' column. Ideally we update it.
                 // For now, let's just mark it read or ignore.
                 // Actually, let's just update read status as a proxy or leave it.
            }
            (StatusCode::OK, Json(serde_json::json!({"id": id}))).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct ListPapersQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub venue_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub is_read: Option<bool>,
}

pub async fn list_papers(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(q): Query<ListPapersQuery>,
) -> impl IntoResponse {
    let filter = crate::domain::prkb::models::PaperFilter {
        venue_id: q.venue_id,
        author_id: q.author_id,
        is_read: q.is_read,
    };
    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);

    match state.repo.list_papers(filter, limit, offset).await {
        Ok(papers) => (StatusCode::OK, Json(papers)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdatePaperRequest {
    pub is_read: Option<bool>,
    pub state: Option<String>,
}

pub async fn list_venues(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    match state.repo.list_venues().await {
        Ok(venues) => (StatusCode::OK, Json(venues)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
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
             return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response();
        }
    }
    if let Some(s) = payload.state {
        if let Err(e) = state.repo.update_paper_state(id, s).await {
             return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response();
        }
    }
    (StatusCode::OK, Json(serde_json::json!({"status": "updated"}))).into_response()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/prkb/feeds", get(list_feeds).post(create_feed))
        .route("/api/prkb/feeds/:id", delete(delete_feed))
        .route("/api/prkb/inbox", get(get_inbox))
        .route("/api/prkb/inbox/:id", patch(update_inbox_item))
        .route("/api/prkb/publications", get(get_publications))
        .route("/api/prkb/venues", get(list_venues))
        .route("/api/prkb/fetch", post(fetch_feeds))
        .route("/api/prkb/papers", get(list_papers).post(save_paper))
        .route("/api/prkb/papers/:id", patch(update_paper))
}
