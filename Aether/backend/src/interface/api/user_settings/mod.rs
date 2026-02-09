use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde_json::Value;

use crate::{
    domain::dtos::user::UserSettingsDto,
    interface::{
        state::AppState,
        api::auth::AuthenticatedUser,
    },
    infrastructure::persistence::repositories::settings::SettingsRepository,
};

#[utoipa::path(
    get,
    path = "/api/users/settings/{module_key}",
    tag = "User Settings",
    responses(
        (status = 200, description = "Get user module settings", body = Value),
        (status = 404, description = "Settings not found"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_user_settings(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(module_key): Path<String>,
) ->  Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let settings = SettingsRepository::get_settings(&state.repo.db, user.id, &module_key)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(settings.unwrap_or(serde_json::json!({}))))
}

#[utoipa::path(
    put,
    path = "/api/users/settings/{module_key}",
    tag = "User Settings",
    request_body = Value,
    responses(
        (status = 200, description = "Update user module settings", body = Value),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn update_user_settings(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(module_key): Path<String>,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let updated = SettingsRepository::update_settings(&state.repo.db, user.id, &module_key, payload)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(updated))
}

pub fn router() -> axum::Router<AppState> {
    use axum::routing::get;
    axum::Router::new()
        .route("/api/users/settings/:module_key", 
            get(get_user_settings).put(update_user_settings)
        )
}
