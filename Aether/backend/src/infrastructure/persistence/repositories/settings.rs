use crate::domain::dtos::user::UserSettingsDto;
use crate::infrastructure::persistence::entities::user_module_settings;
use crate::infrastructure::persistence::entities::user_module_settings::Entity as UserModuleSettings;
use sea_orm::*;
use serde_json::Value;
use uuid::Uuid;

pub struct SettingsRepository;

impl SettingsRepository {
    pub async fn get_settings(
        db: &DatabaseConnection,
        user_id: Uuid,
        module_key: &str,
    ) -> Result<Option<Value>, DbErr> {
        let settings = UserModuleSettings::find()
            .filter(user_module_settings::Column::UserId.eq(user_id))
            .filter(user_module_settings::Column::ModuleKey.eq(module_key))
            .one(db)
            .await?;

        Ok(settings.map(|s| s.settings))
    }

    pub async fn update_settings(
        db: &DatabaseConnection,
        user_id: Uuid,
        module_key: &str,
        settings: Value,
    ) -> Result<Value, DbErr> {
        let existing = UserModuleSettings::find()
            .filter(user_module_settings::Column::UserId.eq(user_id))
            .filter(user_module_settings::Column::ModuleKey.eq(module_key))
            .one(db)
            .await?;

        match existing {
            Some(model) => {
                let mut active: user_module_settings::ActiveModel = model.into();
                active.settings = Set(settings.clone());
                active.updated_at = Set(chrono::Utc::now());
                let updated = active.update(db).await?;
                Ok(updated.settings)
            }
            None => {
                let active = user_module_settings::ActiveModel {
                    user_id: Set(user_id),
                    module_key: Set(module_key.to_string()),
                    settings: Set(settings.clone()),
                    updated_at: Set(chrono::Utc::now()),
                    ..Default::default()
                };
                let inserted = active.insert(db).await?;
                Ok(inserted.settings)
            }
        }
    }
}
