use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, ActiveValue, ColumnTrait, QueryFilter, ActiveModelTrait};
use crate::infrastructure::persistence::entities::system_setting;
use serde_json::Value;

#[derive(Clone)]
pub struct SystemSettingsRepository {
    db: Arc<DatabaseConnection>,
}

impl SystemSettingsRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn get(&self, key: &str) -> Option<Value> {
        system_setting::Entity::find_by_id(key)
            .one(self.db.as_ref())
            .await
            .ok()
            .flatten()
            .map(|m| m.value)
    }

    pub async fn get_int(&self, key: &str, default: i64) -> i64 {
        match self.get(key).await {
            Some(Value::Number(n)) => n.as_i64().unwrap_or(default),
            Some(Value::String(s)) => s.parse().unwrap_or(default), // Handle string storage case
            _ => default,
        }
    }

    pub async fn set(&self, key: &str, value: Value) -> Result<(), sea_orm::DbErr> {
        let db = self.db.as_ref();
        
        let existing = system_setting::Entity::find_by_id(key).one(db).await?;

        match existing {
            Some(model) => {
                let mut active: system_setting::ActiveModel = model.into();
                active.value = ActiveValue::Set(value);
                active.updated_at = ActiveValue::Set(chrono::Utc::now());
                active.update(db).await?;
            },
            None => {
                let active = system_setting::ActiveModel {
                    key: ActiveValue::Set(key.to_string()),
                    value: ActiveValue::Set(value),
                    description: ActiveValue::Set(None),
                    updated_at: ActiveValue::Set(chrono::Utc::now()),
                };
                active.insert(db).await?;
            }
        }
        Ok(())
    }
}
