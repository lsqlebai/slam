use sea_orm::{Database, DatabaseConnection};

pub struct Repository {
    pub(crate) conn: DatabaseConnection,
}

impl Repository {
    pub async fn new(database_path: &str) -> Result<Self, String> {
        let url = format!("sqlite://{}?mode=rwc", database_path);
        let conn = Database::connect(url).await.map_err(|e| format!("连接数据库失败: {}", e))?;
        let dao = Self { conn };
        dao.init_schema().await?;
        dao.write_check().await?;
        Ok(dao)
    }
}

mod schema;
mod sport;
mod user;
mod compat;
