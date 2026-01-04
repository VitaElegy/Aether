use std::sync::Arc;
use axum::extract::FromRef;
use crate::domain::ports::{
    AuthService, CommentRepository, ContentRepository, ExportService,
    KnowledgeBaseRepository, MemoRepository, TagRepository, UserRepository,
};
use crate::infrastructure::persistence::postgres::PostgresRepository;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<PostgresRepository>,
    pub auth_service: Arc<dyn AuthService>,
    pub export_service: Arc<dyn ExportService>,
}

impl FromRef<AppState> for Arc<dyn AuthService> {
    fn from_ref(state: &AppState) -> Self {
        state.auth_service.clone()
    }
}

impl FromRef<AppState> for Arc<dyn UserRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn UserRepository>
    }
}

impl FromRef<AppState> for Arc<dyn ContentRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn ContentRepository>
    }
}

impl FromRef<AppState> for Arc<dyn CommentRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn CommentRepository>
    }
}

impl FromRef<AppState> for Arc<dyn MemoRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn MemoRepository>
    }
}

impl FromRef<AppState> for Arc<dyn KnowledgeBaseRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn KnowledgeBaseRepository>
    }
}

impl FromRef<AppState> for Arc<dyn TagRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn TagRepository>
    }
}

impl FromRef<AppState> for Arc<dyn ExportService> {
    fn from_ref(state: &AppState) -> Self {
        state.export_service.clone()
    }
}
