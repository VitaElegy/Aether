use sea_orm::DatabaseConnection;
use std::env;
use std::sync::Arc;

use crate::domain::graph_service::GraphService;
use crate::domain::indexer_service::IndexerService;
use crate::domain::kb::SchemaRegistry;
use crate::domain::permission_service::PermissionService;
use crate::domain::ports::{
    ArticleRepository, CommentRepository, GraphRepository, KnowledgeBaseRepository, MemoRepository,
    NodeRepository, UserRepository, VrkbRepository,
};
use crate::infrastructure::auth::jwt_service::Arg2JwtAuthService;
use crate::infrastructure::dictionary::loader::DictionaryLoader;
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::repositories::system_settings_repository::SystemSettingsRepository;
use crate::infrastructure::services::arxiv::ArxivService;
use crate::infrastructure::services::asset_manager::AssetManager;
use crate::infrastructure::services::backup_service::BackupService;
use crate::infrastructure::services::export_service::DataExportService;
use crate::infrastructure::services::portability::assets::AssetsPortabilityProvider;
use crate::infrastructure::services::portability::default::DefaultPortabilityProvider;
use crate::infrastructure::services::portability::english::EnglishPortabilityProvider;
use crate::infrastructure::services::portability::vrkb::VrkbPortabilityProvider;
use crate::infrastructure::services::portability_service::PortabilityService;
use crate::infrastructure::services::rss::RssService;
use crate::infrastructure::storage::service::AssetStorageService;
use crate::interface::state::AppState;

pub async fn init_app_state(db: DatabaseConnection) -> AppState {
    tracing::info!("Initializing Services...");

    // Repositories
    let repo = Arc::new(PostgresRepository::new(db.clone()));

    // Services
    let auth_service = Arc::new(Arg2JwtAuthService::new(
        repo.clone() as Arc<dyn UserRepository>,
        env::var("JWT_SECRET").expect("JWT_SECRET environment variable must be set"),
    ));

    let export_service = Arc::new(DataExportService::new(
        repo.clone() as Arc<dyn ArticleRepository>,
        repo.clone() as Arc<dyn MemoRepository>,
        repo.clone() as Arc<dyn CommentRepository>,
    ));

    let permission_service = PermissionService::new(repo.clone());

    let indexer_service = Arc::new(IndexerService::new(db.clone()));

    let graph_service = Arc::new(GraphService::new(repo.clone() as Arc<dyn GraphRepository>));

    let asset_storage = Arc::new(AssetStorageService::new(
        repo.clone() as Arc<dyn VrkbRepository>,
        "uploads".to_string(),
    ));

    let asset_manager = Arc::new(AssetManager::new(
        repo.clone() as Arc<dyn NodeRepository>,
        repo.clone() as Arc<dyn ArticleRepository>,
        repo.clone() as Arc<dyn KnowledgeBaseRepository>,
        Arc::new(permission_service.clone()),
        ".".to_string(),
    ));

    let backup_service = Arc::new(BackupService::new(
        repo.clone() as Arc<dyn ArticleRepository>,
        repo.clone() as Arc<dyn crate::domain::ports::KnowledgeBaseRepository>,
        repo.clone() as Arc<dyn NodeRepository>,
        asset_manager.clone(),
        ".".to_string(),
    ));

    let mut portability_service = PortabilityService::new();

    // Register English Provider (Standard)
    portability_service.register_provider(Arc::new(EnglishPortabilityProvider::new(
        repo.clone() as Arc<dyn crate::domain::ports::VocabularyRepository>,
        repo.clone() as Arc<dyn ArticleRepository>,
        repo.clone() as Arc<dyn crate::domain::ports::KnowledgeBaseRepository>,
        backup_service.clone(),
    )));
    for alias in ["english", "english_v1_std", "vocabulary", "vocabulary_std"] {
        portability_service.register_alias(alias, "english_v1");
    }

    // Register Default Provider
    portability_service.register_provider(Arc::new(DefaultPortabilityProvider::new(
        backup_service.clone(),
    )));

    // Register Assets Provider
    portability_service.register_provider(Arc::new(AssetsPortabilityProvider::new(
        repo.clone() as Arc<dyn ArticleRepository>,
        repo.clone() as Arc<dyn crate::domain::ports::KnowledgeBaseRepository>,
        repo.clone() as Arc<dyn NodeRepository>,
        ".".to_string(),
    )));
    for alias in ["assets", "assets_v1"] {
        portability_service.register_alias(alias, "assets_v1");
    }

    // Register VRKB Provider (VRKB-10)
    portability_service.register_provider(Arc::new(VrkbPortabilityProvider::new(
        repo.clone() as Arc<dyn VrkbRepository>,
    )));
    for alias in ["vrkb_std", "vulnerability_research"] {
        portability_service.register_alias(alias, "vrkb");
    }

    // Map current non-specialized KB renderers to the default provider until dedicated providers exist.
    for alias in [
        "memo",
        "memo_std",
        "memo_v1",
        "prkb",
        "math",
        "math_std",
        "math_v1",
        "math_v1_std",
        "math_v3",
    ] {
        portability_service.register_alias(alias, "default");
    }

    let portability_service = Arc::new(portability_service);

    let arxiv_service = Arc::new(ArxivService::new());
    let rss_service = Arc::new(RssService::new());

    // Dictionary (Heavy Load)
    let dictionary = DictionaryLoader::new("data/dictionary");
    let dictionary_cache = moka::future::Cache::builder()
        .max_capacity(1000)
        .time_to_live(std::time::Duration::from_secs(3600))
        .build();

    // Schema Registry
    let schema_registry = SchemaRegistry::new();
    schema_registry.register(
        "markdown",
        crate::domain::kb::schemas::markdown::MarkdownSchema,
    );
    schema_registry.register("math_block", crate::domain::kb::schemas::math::MathSchema);
    schema_registry.register("paper", crate::domain::kb::schemas::paper_v1::PaperSchema);

    // Register Asset Schemas
    schema_registry.register(
        "image_asset",
        crate::domain::kb::schemas::assets::ImageAssetSchema,
    );
    schema_registry.register(
        "pdf_asset",
        crate::domain::kb::schemas::assets::PdfAssetSchema,
    );
    schema_registry.register(
        "file_asset",
        crate::domain::kb::schemas::assets::FileAssetSchema,
    );
    schema_registry.register(
        "ip_asset",
        crate::domain::kb::schemas::assets::IpAssetSchema,
    );
    schema_registry.register(
        "credential_stub",
        crate::domain::kb::schemas::assets::CredentialStubSchema,
    );
    schema_registry.register(
        "snippet_asset",
        crate::domain::kb::schemas::assets::SnippetAssetSchema,
    );
    schema_registry.register(
        "domain_asset",
        crate::domain::kb::schemas::assets::DomainAssetSchema,
    );

    tracing::info!("KB Schema Registry initialized (types: markdown, math_block, paper, assets, snippet_asset, domain_asset)");

    let system_settings_repository = Arc::new(SystemSettingsRepository::new(Arc::new(db.clone())));

    // Math Service (MATH-01 through MATH-06)
    let math_service = Arc::new(crate::domain::math::service::MathService::new());
    tracing::info!("Math Service initialized (formal object model + graph semantics)");

    AppState {
        repo,
        auth_service,
        export_service,
        permission_service,
        dictionary,
        dictionary_cache,
        indexer_service,
        graph_service,
        asset_storage,
        asset_manager,
        backup_service,
        portability_service,
        schema_registry,
        arxiv_service,
        rss_service,
        system_settings_repository,
        math_service,
    }
}
