use sea_orm::{Statement, DbBackend, ConnectionTrait};
use super::Repository;

impl Repository {
    pub(crate) async fn exec_batch(&self, sql: &str) -> Result<(), String> {
        for stmt in sql.split(';') {
            let s = stmt.trim();
            if s.is_empty() { continue; }
            self.conn
                .execute(Statement::from_string(DbBackend::Sqlite, s.to_string()))
                .await
                .map_err(|e| format!("执行SQL失败: {}", e))?;
        }
        Ok(())
    }

    pub(crate) async fn init_schema(&self) -> Result<(), String> {
        let create_sql = r#"
        CREATE TABLE IF NOT EXISTS sports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uid INTEGER NOT NULL DEFAULT 0,
            type TEXT NOT NULL,
            start_time INTEGER NOT NULL,
            calories INTEGER NOT NULL,
            distance_meter INTEGER NOT NULL,
            duration_second INTEGER NOT NULL,
            heart_rate_avg INTEGER NOT NULL,
            heart_rate_max INTEGER NOT NULL,
            pace_average TEXT NOT NULL,
            extra TEXT NOT NULL,
            tracks TEXT NOT NULL,
            CHECK (json_valid(extra)),
            CHECK (json_valid(tracks))
        );
        CREATE INDEX IF NOT EXISTS idx_sports_start_time ON sports(start_time);

        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            password TEXT NOT NULL,
            nickname TEXT NOT NULL DEFAULT '',
            avatar TEXT NOT NULL DEFAULT ''
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_users_name ON users(name);
        CREATE TABLE IF NOT EXISTS avatars (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uid INTEGER NOT NULL UNIQUE,
            data TEXT NOT NULL,
            mime TEXT NOT NULL DEFAULT 'image/jpeg',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_avatars_uid ON avatars(uid);
        "#;
        self.exec_batch(create_sql).await?;
        // 兼容历史列添加
        let _ = self.exec_batch("ALTER TABLE users ADD COLUMN nickname TEXT NOT NULL DEFAULT '';\n").await;
        let _ = self.exec_batch("ALTER TABLE users ADD COLUMN avatar TEXT NOT NULL DEFAULT '';\n").await;
        Ok(())
    }

    pub(crate) async fn write_check(&self) -> Result<(), String> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS __healthcheck (id INTEGER PRIMARY KEY AUTOINCREMENT, n INTEGER NOT NULL);
        "#;
        self.exec_batch(sql).await?;
        self.exec_batch("INSERT INTO __healthcheck (n) VALUES (1);").await?;
        self.exec_batch("DELETE FROM __healthcheck;").await?;
        Ok(())
    }
}
