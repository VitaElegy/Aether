use sea_orm::{DatabaseConnection};

#[derive(Clone)]
pub struct PostgresRepository {
    pub db: DatabaseConnection, // Public for sub-modules to access
}

impl PostgresRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
