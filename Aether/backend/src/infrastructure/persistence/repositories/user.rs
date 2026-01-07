use async_trait::async_trait;
use sea_orm::*;
use chrono::Utc;
use crate::domain::models::{User, UserId};
use crate::domain::ports::{UserRepository, RepositoryError};
use crate::infrastructure::persistence::postgres::PostgresRepository;
use crate::infrastructure::persistence::entities::user;

#[async_trait]
impl UserRepository for PostgresRepository {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        let model = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(model.map(|m| User {
            id: UserId(m.id),
            username: m.username,
            email: m.email,
            display_name: m.display_name,
            bio: m.bio,
            avatar_url: m.avatar_url,
            password_hash: m.password_hash,
            permissions: m.permissions as u64,
        }))
    }

    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepositoryError> {
        let model = user::Entity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        Ok(model.map(|m| User {
            id: UserId(m.id),
            username: m.username,
            email: m.email,
            display_name: m.display_name,
            bio: m.bio,
            avatar_url: m.avatar_url,
            password_hash: m.password_hash,
            permissions: m.permissions as u64,
        }))
    }

    async fn save(&self, u: User) -> Result<UserId, RepositoryError> {
        let model = user::ActiveModel {
            created_at: Set(Utc::now().into()), // Added
            id: Set(u.id.0),
            username: Set(u.username),
            email: Set(u.email),
            display_name: Set(u.display_name),
            bio: Set(u.bio),
            avatar_url: Set(u.avatar_url),
            password_hash: Set(u.password_hash),
            permissions: Set(u.permissions as i64),
        };
        user::Entity::insert(model)
             .on_conflict(
                sea_orm::sea_query::OnConflict::column(user::Column::Id)
                    .update_columns([
                        user::Column::Username, user::Column::Email, user::Column::DisplayName,
                        user::Column::Bio, user::Column::AvatarUrl, user::Column::Permissions
                    ])
                    .to_owned()
            )
            .exec(&self.db).await.map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        Ok(u.id)
    }
}
