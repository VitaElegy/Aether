use axum::Router;
use crate::interface::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
