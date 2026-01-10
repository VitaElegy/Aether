use std::sync::Arc;
use axum::extract::FromRef;
use crate::domain::ports::{
    AuthService, CommentRepository, ArticleRepository, ExportService,
    MemoRepository, UserRepository, VocabularyRepository, NodeRepository
};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::dictionary::loader::DictionaryLoader;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<PostgresRepository>,
    pub auth_service: Arc<dyn AuthService>,
    pub export_service: Arc<dyn ExportService>,
    pub permission_service: crate::domain::permission_service::PermissionService<PostgresRepository>,
    pub dictionary: DictionaryLoader,
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

impl FromRef<AppState> for Arc<dyn ArticleRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn ArticleRepository>
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

impl FromRef<AppState> for Arc<dyn VocabularyRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn VocabularyRepository>
    }
}

impl FromRef<AppState> for Arc<dyn NodeRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn NodeRepository>
    }
}

impl FromRef<AppState> for Arc<dyn ExportService> {
    fn from_ref(state: &AppState) -> Self {
        state.export_service.clone()
    }
}

impl FromRef<AppState> for Arc<dyn crate::domain::ports::KnowledgeBaseRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn crate::domain::ports::KnowledgeBaseRepository>
    }
}

impl FromRef<AppState> for crate::domain::permission_service::PermissionService<PostgresRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.permission_service.clone()
    }
}
