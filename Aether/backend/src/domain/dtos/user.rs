use serde::{Deserialize, Serialize};
use serde_json::Value; // JSON types
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSettingsDto {
    pub user_id: Uuid,
    pub module_key: String,
    pub settings: Value,
}
