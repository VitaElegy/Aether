use axum::{
    Json, extract::{State, FromRequestParts}, response::IntoResponse, http::{StatusCode, request::Parts, header::AUTHORIZATION},
    extract::FromRef,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::domain::{
    ports::{AuthService, UserRepository},
    models::{User, permissions},
};
use uuid::Uuid;
use crate::infrastructure::auth::jwt_service::hash_password;

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Clone, Copy, Debug)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub permissions: u64,
}

#[allow(dead_code)]
impl AuthenticatedUser {
    pub fn has_permission(&self, required_perm: u64) -> bool {
        (self.permissions & required_perm) == required_perm
    }

    pub fn can_comment(&self) -> bool {
        self.has_permission(permissions::COMMENT)
    }

    pub fn can_create_post(&self) -> bool {
        // Mapped to CREATE_NODE for Phase 1
        self.has_permission(permissions::CREATE_NODE)
    }

    pub fn can_manage_todos(&self) -> bool {
        // Mapped to ADMIN or specific TODO perm if we add it. For now ADMIN.
        self.has_permission(permissions::ADMIN)
    }

    pub fn is_admin(&self) -> bool {
        self.has_permission(permissions::ADMIN)
    }
}

#[axum::async_trait]
impl FromRequestParts<crate::interface::state::AppState> for AuthenticatedUser
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, state: &crate::interface::state::AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get(AUTHORIZATION)
            .ok_or((StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Missing bearer token" }))))?
            .to_str()
            .map_err(|_| (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Invalid token header" }))))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Invalid token format" }))));
        }
        let token = &auth_header[7..];

        let auth_service: Arc<dyn AuthService> = FromRef::from_ref(state);

        match auth_service.verify_token(token) {
            Ok(claims) => {
                let id = Uuid::parse_str(&claims.sub).map_err(|_| (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Invalid user ID in token" }))))?;
                
                let user_repo: Arc<dyn UserRepository> = FromRef::from_ref(state);
                match user_repo.find_by_id(&crate::domain::models::UserId(id)).await {
                    Ok(Some(user)) => Ok(AuthenticatedUser {
                        id: user.id.0,
                        permissions: claims.perms, // Or refresh perms from DB if desired
                    }),
                    Ok(None) => Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "User no longer exists" })))),
                    Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Database error checking user" })))),
                }
            },
            Err(_) => Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Invalid token" })))),
        }
    }
}

pub struct MaybeAuthenticatedUser(pub Option<AuthenticatedUser>);

#[axum::async_trait]
impl FromRequestParts<crate::interface::state::AppState> for MaybeAuthenticatedUser
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, state: &crate::interface::state::AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get(AUTHORIZATION);

        match auth_header {
            Some(header_value) => {
                 let header_str = header_value.to_str().map_err(|_| (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Invalid token header" }))))?;
                 if !header_str.starts_with("Bearer ") {
                     return Ok(MaybeAuthenticatedUser(None));
                 }
                 let token = &header_str[7..];
                 let auth_service: Arc<dyn AuthService> = FromRef::from_ref(state);
                 match auth_service.verify_token(token) {
                    Ok(claims) => {
                        let id = Uuid::parse_str(&claims.sub).unwrap_or_default();
                        let user_repo: Arc<dyn UserRepository> = FromRef::from_ref(state);
                        match user_repo.find_by_id(&crate::domain::models::UserId(id)).await {
                            Ok(Some(user)) => Ok(MaybeAuthenticatedUser(Some(AuthenticatedUser {
                                id: user.id.0,
                                permissions: claims.perms,
                            }))),
                            _ => Ok(MaybeAuthenticatedUser(None)), // User invalid or DB error -> verification failed
                        }
                    },
                    Err(_) => Ok(MaybeAuthenticatedUser(None)), 
                 }
            },
            None => Ok(MaybeAuthenticatedUser(None)),
        }
    }
}

pub async fn login_handler(
    State(state): State<crate::interface::state::AppState>, 
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match state.auth_service.authenticate(&payload.username, &payload.password).await {
        Ok(claims) => {
             match state.auth_service.generate_token(&claims) {
                Ok(token) => {
                    let user_id = crate::domain::models::UserId(Uuid::parse_str(&claims.sub).unwrap());
                    let user = state.repo.find_by_id(&user_id).await.unwrap().unwrap();

                    (StatusCode::OK, Json(serde_json::json!({
                        "token": token,
                        "user": user
                    })))
                },
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
             }
        },
        Err(_) => (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Invalid credentials" }))),
    }
}

pub async fn register_handler(
    State(state): State<crate::interface::state::AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    if let Ok(Some(_)) = state.repo.find_by_username(&payload.username).await {
         return (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Username already taken" })));
    }

    let user = User {
        id: crate::domain::models::UserId(Uuid::new_v4()),
        username: payload.username,
        email: payload.email,
        display_name: None,
        bio: None,
        avatar_url: None,
        password_hash: hash_password(&payload.password),
        permissions: 1, 
    };

    match state.repo.save(user).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({ "message": "User created" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    display_name: Option<String>,
    bio: Option<String>,
    avatar_url: Option<String>,
    email: Option<String>,
}

pub async fn get_user_handler(
    State(state): State<crate::interface::state::AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    let user_id = crate::domain::models::UserId(id);
    match state.repo.find_by_id(&user_id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(serde_json::to_value(user).unwrap())),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "User not found" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn update_user_handler(
    State(state): State<crate::interface::state::AppState>,
    auth_user: AuthenticatedUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    if auth_user.id != id {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Cannot update other users" })));
    }

    let user_id = crate::domain::models::UserId(id);

    let mut user = match state.repo.find_by_id(&user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "User not found" }))),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    };

    if let Some(name) = payload.display_name { user.display_name = Some(name); }
    if let Some(bio) = payload.bio { user.bio = Some(bio); }
    if let Some(avatar) = payload.avatar_url { user.avatar_url = Some(avatar); }
    if let Some(email) = payload.email { user.email = email; }

    match state.repo.save(user).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "message": "User updated" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}



pub fn router() -> axum::Router<crate::interface::state::AppState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/register", post(register_handler))

        .route("/api/users/:id", get(get_user_handler).put(update_user_handler))
}
