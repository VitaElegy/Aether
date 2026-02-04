use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
use uuid::Uuid;
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::domain::ports::{UserRepository, PermissionRepository};
use crate::domain::models::User;

pub async fn seed_all(db: &DatabaseConnection, repo: &Arc<PostgresRepository>) {
    tracing::info!("Starting Seeding Process...");
    
    seed_admin_user(repo).await;
    seed_layout_templates(db).await;
    seed_public_group(repo).await;
    seed_public_group(repo).await;
    seed_system_knowledge_base(db, repo).await;
    seed_prkb_feeds(db).await;
    
    tracing::info!("Seeding Process Complete.");
}

async fn seed_admin_user(repo: &Arc<PostgresRepository>) {
    tracing::info!("Checking for admin user...");
    let admin_name = "admin";
    
    // Timeout check
    if let Ok(Some(mut admin)) = tokio::time::timeout(std::time::Duration::from_secs(5), repo.find_by_username(admin_name)).await.unwrap_or(Ok(None)) {
        tracing::info!("Updating admin user...");
        if admin.display_name.is_none() { admin.display_name = Some("Administrator".to_string()); }
        if admin.bio.is_none() { admin.bio = Some("The system architect.".to_string()); }
        let _ = UserRepository::save(repo.as_ref(), admin).await;
    } else {
        tracing::info!("Seeding admin user...");
        let hash = crate::infrastructure::auth::jwt_service::hash_password("admin");
        let admin = User {
            id: crate::domain::models::UserId(Uuid::new_v4()),
            username: admin_name.to_string(),
            email: "admin@aether.io".to_string(),
            display_name: Some("Administrator".to_string()),
            bio: Some("The system architect.".to_string()),
            avatar_url: None,
            password_hash: hash,
            permissions: u64::MAX,
            experience: None,
        };
        let _ = UserRepository::save(repo.as_ref(), admin).await;
    }
}

async fn seed_layout_templates(db: &DatabaseConnection) {
    use crate::infrastructure::persistence::entities::layout_template;
    
    tracing::info!("Seeding Layout Templates...");
    
    let templates = vec![
        ("default", "Blog Standard", "Standard single-column layout for general writing.", "bg-gradient-to-br from-blue-500 to-cyan-500", vec!["General", "Writing"]),
        ("math_v3", "Math Manuscript V3", "Latex-heavy two-column layout for mathematical proofs.", "bg-gradient-to-br from-indigo-500 to-purple-500", vec!["Math", "Academic"]),
        ("vrkb", "Vulnerability Research", "Kanban-style board for tracking finding and assets.", "bg-gradient-to-br from-orange-500 to-red-500", vec!["Security", "Workflow"]),
        ("english", "English Learning", "Vocabulary, dictionary, and flashcard workspace.", "bg-gradient-to-br from-green-400 to-teal-500", vec!["Language", "Study"]),
        ("prkb", "Paper Research", "Track ArXiv feeds, read papers, and manage research library.", "bg-gradient-to-br from-purple-600 to-pink-500", vec!["Research", "Academic"]),
        ("memo", "Memo Board", "Grid layout for quick notes and thoughts.", "bg-gradient-to-br from-yellow-400 to-orange-400", vec!["Personal", "Notes"]),
        ("admin_system", "System Control", "Protected interface for system administration.", "bg-gradient-to-br from-gray-700 to-black", vec!["Admin", "System"]),
    ];

    for (rid, title, desc, thumb, tags) in templates {
        tracing::info!("Processing template: {} ({})", title, rid);
        // Robust Upsert: Find by RendererID first
        let existing = layout_template::Entity::find()
            .filter(layout_template::Column::RendererId.eq(rid))
            .one(db)
            .await
            .unwrap_or(None);

        if let Some(model) = existing {
            // Update mode
            let mut active: layout_template::ActiveModel = model.into();
            active.title = Set(title.to_string());
            active.description = Set(desc.to_string());
            active.thumbnail = Set(Some(thumb.to_string()));
            active.updated_at = Set(chrono::Utc::now().into());
            if let Err(e) = active.update(db).await {
                tracing::warn!("Failed to update template {}: {}", rid, e);
            }
        } else {
            // Insert mode
            tracing::info!("Creating Template: {}", title);
            let active = layout_template::ActiveModel {
                 id: Set(Uuid::new_v4()),
                 renderer_id: Set(rid.to_string()),
                 title: Set(title.to_string()),
                 description: Set(desc.to_string()),
                 thumbnail: Set(Some(thumb.to_string())),
                 tags: Set(Some(serde_json::to_value(tags).unwrap())),
                 config: Set(Some(serde_json::json!({}))),
                 created_at: Set(chrono::Utc::now().into()),
                 updated_at: Set(chrono::Utc::now().into()),
             };
             if let Err(e) = active.insert(db).await {
                 tracing::error!("Failed to seed template {}: {}", title, e);
             }
        }
    }
}

async fn seed_public_group(repo: &Arc<PostgresRepository>) {
    // Initialize Public Group
    let public_group_id = Uuid::nil(); 
    // We don't have a check logic in repo easily for groups yet, simply try create.
    // Repo usually handles idempotency or we ignore error.
    match repo.create_group(public_group_id, "public".to_string()).await {
        Ok(_) => tracing::info!("Public group initialized"),
        Err(_) => {}, // Assume exists
    }
}

async fn seed_system_knowledge_base(db: &DatabaseConnection, repo: &Arc<PostgresRepository>) {
    use crate::infrastructure::persistence::entities::knowledge_base;
    
    tracing::info!("Checking for System Admin Knowledge Base...");
    
    // Find admin user first
    let admin_user = repo.find_by_username("admin").await.unwrap().unwrap();
    
    let kb_title = "Admin System";
    let renderer_id = "admin_system";
    
    // Check if exists by renderer_id
    let existing = knowledge_base::Entity::find()
        .filter(knowledge_base::Column::RendererId.eq(renderer_id))
        .one(db)
        .await
        .unwrap_or(None);

    if existing.is_none() {
        tracing::info!("Seeding System Knowledge Base...");
        let active = knowledge_base::ActiveModel {
            id: Set(Uuid::new_v4()),
            author_id: Set(admin_user.id.0),
            title: Set(kb_title.to_string()),
            description: Set(Some("System Administration Workspace".to_string())),
            tags: Set(serde_json::json!(["System", "Admin"])),
            cover_image: Set(Some("https://images.unsplash.com/photo-1550751827-4bd374c3f58b".to_string())),
            cover_offset_y: Set(0),
            renderer_id: Set(Some(renderer_id.to_string())),
            visibility: Set("private".to_string()),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
        };
        if let Err(e) = active.insert(db).await {
             tracing::error!("Failed to seed System KB: {}", e);
        }
    }
}

async fn seed_prkb_feeds(db: &DatabaseConnection) {
    use crate::infrastructure::persistence::entities::prkb_feeds;
    
    tracing::info!("Seeding PRKB Feeds...");
    
    let feeds = vec![
        ("ArXiv Cryptography", "cs.CR", "arxiv"),
        ("Google Project Zero", "https://googleprojectzero.blogspot.com/feeds/posts/default", "rss"),
        ("Sec-Circus", "https://sec-circus.com/feed", "rss"),
        ("USENIX Security", "https://www.usenix.org/rss/conference/usenixsecurity24/rss.xml", "rss"),
        ("Full Disclosure", "https://seclists.org/rss/fulldisclosure.rss", "rss"),
    ];

    for (name, url, ftype) in feeds {
        let existing = prkb_feeds::Entity::find()
            .filter(prkb_feeds::Column::Url.eq(url))
            .one(db)
            .await
            .unwrap_or(None);

        if existing.is_none() {
            tracing::info!("Creating Feed: {}", name);
            let active = prkb_feeds::ActiveModel {
                id: Set(Uuid::new_v4()),
                name: Set(name.to_string()),
                url: Set(url.to_string()),
                feed_type: Set(ftype.to_string()),
                last_fetched_at: Set(None),
                created_at: Set(chrono::Utc::now().into()),
            };
            if let Err(e) = active.insert(db).await {
                tracing::error!("Failed to seed feed {}: {}", name, e);
            }
        }
    }
}
