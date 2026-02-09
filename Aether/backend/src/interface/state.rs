use std::sync::Arc;
use axum::extract::FromRef;
use crate::domain::ports::{
    AuthService, CommentRepository, ArticleRepository, ExportService,
    MemoRepository, UserRepository, VocabularyRepository, NodeRepository, VrkbRepository,

};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::dictionary::loader::DictionaryLoader;
use crate::domain::indexer_service::IndexerService;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<PostgresRepository>,
    pub auth_service: Arc<dyn AuthService>,
    pub export_service: Arc<dyn ExportService>,
    pub permission_service: crate::domain::permission_service::PermissionService<PostgresRepository>,
    pub dictionary: DictionaryLoader,
    pub dictionary_cache: moka::future::Cache<String, String>, // JSON serialized entry
    pub indexer_service: Arc<IndexerService>,
    pub graph_service: Arc<crate::domain::graph_service::GraphService>,
    pub asset_storage: Arc<crate::infrastructure::storage::service::AssetStorageService>,
    pub schema_registry: crate::domain::kb::SchemaRegistry,
    pub arxiv_service: Arc<crate::infrastructure::services::arxiv::ArxivService>,
    pub rss_service: Arc<crate::infrastructure::services::rss::RssService>,
    pub asset_manager: Arc<crate::infrastructure::services::asset_manager::AssetManager>,
    pub backup_service: Arc<crate::infrastructure::services::backup_service::BackupService>,
    pub portability_service: Arc<crate::infrastructure::services::portability_service::PortabilityService>,
    pub system_settings_repository: Arc<crate::infrastructure::persistence::repositories::system_settings_repository::SystemSettingsRepository>,
}

impl FromRef<AppState> for Arc<dyn AuthService> {
    fn from_ref(state: &AppState) -> Self {
        state.auth_service.clone()
    }
}

impl FromRef<AppState> for Arc<IndexerService> {
    fn from_ref(state: &AppState) -> Self {
        state.indexer_service.clone()
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

impl FromRef<AppState> for Arc<dyn VrkbRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone() as Arc<dyn VrkbRepository>
    }
}

impl FromRef<AppState> for Arc<crate::infrastructure::storage::service::AssetStorageService> {
    fn from_ref(state: &AppState) -> Self {
        state.asset_storage.clone()
    }
}

impl FromRef<AppState> for Arc<crate::infrastructure::persistence::repositories::system_settings_repository::SystemSettingsRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.system_settings_repository.clone()
    }
}

impl FromRef<AppState> for Arc<crate::infrastructure::services::asset_manager::AssetManager> {
    fn from_ref(state: &AppState) -> Self {
        state.asset_manager.clone()
    }
}
