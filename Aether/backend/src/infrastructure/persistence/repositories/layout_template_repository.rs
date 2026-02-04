use super::super::entities::layout_template;
use crate::domain::models::LayoutTemplate;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;

pub struct LayoutTemplateRepository;

impl LayoutTemplateRepository {
    pub async fn create(db: &DbConn, template: LayoutTemplate) -> Result<LayoutTemplate, DbErr> {
        let active_model = layout_template::ActiveModel {
            id: Set(template.id),
            renderer_id: Set(template.renderer_id.clone()),
            title: Set(template.title.clone()),
            description: Set(template.description.clone()),
            thumbnail: Set(template.thumbnail.clone()),
            tags: Set(Some(serde_json::to_value(template.tags.clone()).unwrap())),
            config: Set(Some(template.config.clone())),
            created_at: Set(template.created_at),
            updated_at: Set(template.updated_at),
        };

        active_model.insert(db).await?;
        
        Ok(template)
    }

    pub async fn list(db: &DbConn) -> Result<Vec<LayoutTemplate>, DbErr> {
        let models = layout_template::Entity::find().all(db).await?;
        
        let templates = models.into_iter().map(|m| LayoutTemplate {
            id: m.id,
            renderer_id: m.renderer_id,
            title: m.title,
            description: m.description,
            thumbnail: m.thumbnail,
            tags: m.tags.map(|t| serde_json::from_value(t).unwrap_or_default()).unwrap_or_default(),
            config: m.config.unwrap_or(serde_json::json!({})),
            created_at: m.created_at,
            updated_at: m.updated_at,
        }).collect();

        Ok(templates)
    }

    pub async fn update(db: &DbConn, id: Uuid, template: LayoutTemplate) -> Result<LayoutTemplate, DbErr> {
        let mut active_model: layout_template::ActiveModel = layout_template::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(id.to_string()))?
            .into();

        active_model.renderer_id = Set(template.renderer_id.clone());
        active_model.title = Set(template.title.clone());
        active_model.description = Set(template.description.clone());
        active_model.thumbnail = Set(template.thumbnail.clone());
        active_model.tags = Set(Some(serde_json::to_value(template.tags.clone()).unwrap()));
        active_model.config = Set(Some(template.config.clone()));
        active_model.updated_at = Set(Utc::now());

        active_model.update(db).await?;

        Ok(template)
    }

    pub async fn delete(db: &DbConn, id: Uuid) -> Result<(), DbErr> {
        layout_template::Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }
    
    #[allow(dead_code)]
    pub async fn count(db: &DbConn) -> Result<u64, DbErr> {
        layout_template::Entity::find().count(db).await
    }
}
