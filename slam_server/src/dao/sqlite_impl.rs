use async_trait::async_trait;
use rusqlite::{params, Connection, OpenFlags};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use serde_json;

use crate::model::sport::{Sport, Swimming, Track, SportType};
use crate::model::user::{User, UserInfo};
use super::idl::{SportDao, UserDao};

pub struct SqliteImpl {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteImpl {
    pub async fn new(database_path: &str) -> Result<Self, String> {
        Self::new_sync(database_path)
    }

    pub fn new_sync(database_path: &str) -> Result<Self, String> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
        let manager = SqliteConnectionManager::file(database_path).with_flags(flags);
        let pool = Pool::new(manager).map_err(|e| format!("创建连接池失败: {}", e))?;
        let dao = Self { pool };
        {
            let conn = dao.get_conn()?;
            dao.init_schema_with_conn(&conn)?;
            dao.write_check(&conn)?;
        }
        Ok(dao)
    }

    fn get_conn(&self) -> Result<PooledConnection<SqliteConnectionManager>, String> {
        self.pool.get().map_err(|e| format!("获取连接失败: {}", e))
    }

    fn init_schema_with_conn(&self, conn: &Connection) -> Result<(), String> {
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
        conn.execute_batch(create_sql)
            .map_err(|e| format!("建表失败: {}", e))?;
        let _ = conn.execute("ALTER TABLE users ADD COLUMN nickname TEXT NOT NULL DEFAULT ''", params![]);
        let _ = conn.execute("ALTER TABLE users ADD COLUMN avatar TEXT NOT NULL DEFAULT ''", params![]);
        Ok(())
    }

    fn write_check(&self, conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS __healthcheck (id INTEGER PRIMARY KEY AUTOINCREMENT, n INTEGER NOT NULL);
            "#,
        ).map_err(|e| format!("写入检查表创建失败: {}", e))?;
        conn.execute("INSERT INTO __healthcheck (n) VALUES (1)", params![])
            .map_err(|e| format!("写入检查失败: {}", e))?;
        conn.execute("DELETE FROM __healthcheck", params![])
            .map_err(|e| format!("写入检查失败: {}", e))?;
        Ok(())
    }
}

#[async_trait]
impl SportDao for SqliteImpl {
    async fn insert(&self, uid: i32, sport: Sport) -> Result<(), String> {
        let extra_json = serde_json::to_string(&sport.extra)
            .map_err(|e| format!("extra 序列化失败: {}", e))?;
        let tracks_json = serde_json::to_string(&sport.tracks)
            .map_err(|e| format!("tracks 序列化失败: {}", e))?;

        let conn = self.get_conn()?;
        conn
            .execute(
                r#"
                INSERT INTO sports (
                    uid, type, start_time, calories, distance_meter, duration_second,
                    heart_rate_avg, heart_rate_max, pace_average, extra, tracks
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    uid,
                    sport.r#type.as_str(),
                    sport.start_time,
                    sport.calories,
                    sport.distance_meter,
                    sport.duration_second,
                    sport.heart_rate_avg,
                    sport.heart_rate_max,
                    sport.pace_average,
                    extra_json,
                    tracks_json,
                ],
            )
            .map_err(|e| format!("插入失败: {}", e))?;
        Ok(())
    }

    async fn insert_many(&self, uid: i32, sports: Vec<Sport>) -> Result<usize, String> {
        let mut conn = self.get_conn()?;
        let tx = conn.transaction().map_err(|e| format!("开启事务失败: {}", e))?;
        let mut count = 0usize;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO sports (
                    uid, type, start_time, calories, distance_meter, duration_second,
                    heart_rate_avg, heart_rate_max, pace_average, extra, tracks
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            ).map_err(|e| format!("预编译失败: {}", e))?;
            for sport in sports {
                let extra_json = serde_json::to_string(&sport.extra).map_err(|e| e.to_string())?;
                let tracks_json = serde_json::to_string(&sport.tracks).map_err(|e| e.to_string())?;
                stmt.execute(params![
                    uid,
                    sport.r#type.as_str(),
                    sport.start_time,
                    sport.calories,
                    sport.distance_meter,
                    sport.duration_second,
                    sport.heart_rate_avg,
                    sport.heart_rate_max,
                    sport.pace_average,
                    extra_json,
                    tracks_json,
                ]).map_err(|e| format!("插入失败: {}", e))?;
                count += 1;
            }
        }
        tx.commit().map_err(|e| format!("提交事务失败: {}", e))?;
        Ok(count)
    }

    async fn list(&self, uid: i32, page: i32, size: i32) -> Result<Vec<Sport>, String> {
        let safe_size = if size <= 0 { 20 } else { size.min(100) } as i64;
        let safe_page = if page < 0 { 0 } else { page } as i64;
        let offset = safe_page * safe_size;
        let conn = self.get_conn()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, type, start_time, calories, distance_meter, duration_second,
                       heart_rate_avg, heart_rate_max, pace_average, extra, tracks
                FROM sports
                WHERE uid = ?
                ORDER BY start_time DESC
                LIMIT ? OFFSET ?
                "#,
            )
            .map_err(|e| format!("查询失败: {}", e))?;

        let rows = stmt
            .query_map(params![uid, safe_size, offset], |row| {
                let id: i32 = row.get::<_, i64>(0).unwrap_or(0) as i32;
                let type_str: String = row.get(1).unwrap_or_default();
                let start_time: i64 = row.get(2).unwrap_or(0);
                let calories: i32 = row.get::<_, i64>(3).unwrap_or(0) as i32;
                let distance_meter: i32 = row.get::<_, i64>(4).unwrap_or(0) as i32;
                let duration_second: i32 = row.get::<_, i64>(5).unwrap_or(0) as i32;
                let heart_rate_avg: i32 = row.get::<_, i64>(6).unwrap_or(0) as i32;
                let heart_rate_max: i32 = row.get::<_, i64>(7).unwrap_or(0) as i32;
                let pace_average: String = row.get(8).unwrap_or_default();
                let extra_json: String = row.get(9).unwrap_or_else(|_| "{}".to_string());
                let tracks_json: String = row.get(10).unwrap_or_else(|_| "[]".to_string());

                let extra: Swimming = serde_json::from_str(&extra_json).unwrap_or_default();
                let tracks: Vec<Track> = serde_json::from_str(&tracks_json).unwrap_or_default();

                Ok(Sport {
                    id,
                    r#type: SportType::from_str(&type_str),
                    start_time,
                    calories,
                    distance_meter,
                    duration_second,
                    heart_rate_avg,
                    heart_rate_max,
                    pace_average,
                    extra,
                    tracks,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        Ok(result)
    }

    async fn list_by_time_range(&self, uid: i32, start_time: i64, end_time: i64) -> Result<Vec<Sport>, String> {
        let conn = self.get_conn()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, type, start_time, calories, distance_meter, duration_second,
                       heart_rate_avg, heart_rate_max, pace_average, extra, tracks
                FROM sports
                WHERE uid = ? AND start_time BETWEEN ? AND ?
                ORDER BY start_time DESC
                "#,
            )
            .map_err(|e| format!("查询失败: {}", e))?;

        let rows = stmt
            .query_map(params![uid, start_time, end_time], |row| {
                let id: i32 = row.get::<_, i64>(0).unwrap_or(0) as i32;
                let type_str: String = row.get(1).unwrap_or_default();
                let start_time: i64 = row.get(2).unwrap_or(0);
                let calories: i32 = row.get::<_, i64>(3).unwrap_or(0) as i32;
                let distance_meter: i32 = row.get::<_, i64>(4).unwrap_or(0) as i32;
                let duration_second: i32 = row.get::<_, i64>(5).unwrap_or(0) as i32;
                let heart_rate_avg: i32 = row.get::<_, i64>(6).unwrap_or(0) as i32;
                let heart_rate_max: i32 = row.get::<_, i64>(7).unwrap_or(0) as i32;
                let pace_average: String = row.get(8).unwrap_or_default();
                let extra_json: String = row.get(9).unwrap_or_else(|_| "{}".to_string());
                let tracks_json: String = row.get(10).unwrap_or_else(|_| "[]".to_string());

                let extra: Swimming = serde_json::from_str(&extra_json).unwrap_or_default();
                let tracks: Vec<Track> = serde_json::from_str(&tracks_json).unwrap_or_default();

                Ok(Sport {
                    id,
                    r#type: SportType::from_str(&type_str),
                    start_time,
                    calories,
                    distance_meter,
                    duration_second,
                    heart_rate_avg,
                    heart_rate_max,
                    pace_average,
                    extra,
                    tracks,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        Ok(result)
    }

    async fn update(&self, uid: i32, sport: Sport) -> Result<(), String> {
        if sport.id <= 0 { return Err("invalid sport id".to_string()); }
        let extra_json = serde_json::to_string(&sport.extra)
            .map_err(|e| format!("extra 序列化失败: {}", e))?;
        let tracks_json = serde_json::to_string(&sport.tracks)
            .map_err(|e| format!("tracks 序列化失败: {}", e))?;
        let conn = self.get_conn()?;
        let affected = conn
            .execute(
                r#"
                UPDATE sports SET
                    type = ?, start_time = ?, calories = ?, distance_meter = ?, duration_second = ?,
                    heart_rate_avg = ?, heart_rate_max = ?, pace_average = ?, extra = ?, tracks = ?
                WHERE id = ? AND uid = ?
                "#,
                params![
                    sport.r#type.as_str(),
                    sport.start_time,
                    sport.calories,
                    sport.distance_meter,
                    sport.duration_second,
                    sport.heart_rate_avg,
                    sport.heart_rate_max,
                    sport.pace_average,
                    extra_json,
                    tracks_json,
                    sport.id,
                    uid,
                ],
            )
            .map_err(|e| format!("更新失败: {}", e))?;
        if affected == 0 { return Err("记录不存在或无权限".to_string()); }
        Ok(())
    }

    async fn remove(&self, uid: i32, id: i32) -> Result<(), String> {
        if id <= 0 { return Err("invalid sport id".to_string()); }
        let conn = self.get_conn()?;
        let affected = conn
            .execute(
                r#"
                DELETE FROM sports WHERE id = ? AND uid = ?
                "#,
                params![id, uid],
            )
            .map_err(|e| format!("删除失败: {}", e))?;
        if affected == 0 { return Err("记录不存在或无权限".to_string()); }
        Ok(())
    }

    async fn get_by_id(&self, uid: i32, id: i32) -> Result<Option<Sport>, String> {
        if id <= 0 { return Ok(None); }
        let conn = self.get_conn()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, type, start_time, calories, distance_meter, duration_second,
                       heart_rate_avg, heart_rate_max, pace_average, extra, tracks
                FROM sports
                WHERE id = ? AND uid = ?
                LIMIT 1
                "#,
            )
            .map_err(|e| format!("查询失败: {}", e))?;

        let mut rows = stmt
            .query(params![id, uid])
            .map_err(|e| format!("查询失败: {}", e))?;

        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let id: i32 = row.get::<_, i64>(0).unwrap_or(0) as i32;
            let type_str: String = row.get(1).unwrap_or_default();
            let start_time: i64 = row.get(2).unwrap_or(0);
            let calories: i32 = row.get::<_, i64>(3).unwrap_or(0) as i32;
            let distance_meter: i32 = row.get::<_, i64>(4).unwrap_or(0) as i32;
            let duration_second: i32 = row.get::<_, i64>(5).unwrap_or(0) as i32;
            let heart_rate_avg: i32 = row.get::<_, i64>(6).unwrap_or(0) as i32;
            let heart_rate_max: i32 = row.get::<_, i64>(7).unwrap_or(0) as i32;
            let pace_average: String = row.get(8).unwrap_or_default();
            let extra_json: String = row.get(9).unwrap_or_else(|_| "{}".to_string());
            let tracks_json: String = row.get(10).unwrap_or_else(|_| "[]".to_string());

            let extra: Swimming = serde_json::from_str(&extra_json).unwrap_or_default();
            let tracks: Vec<Track> = serde_json::from_str(&tracks_json).unwrap_or_default();

            Ok(Some(Sport {
                id,
                r#type: SportType::from_str(&type_str),
                start_time,
                calories,
                distance_meter,
                duration_second,
                heart_rate_avg,
                heart_rate_max,
                pace_average,
                extra,
                tracks,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_first(&self, uid: i32) -> Result<Option<Sport>, String> {
        let conn = self.get_conn()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, type, start_time, calories, distance_meter, duration_second,
                       heart_rate_avg, heart_rate_max, pace_average, extra, tracks
                FROM sports
                WHERE uid = ?
                ORDER BY start_time ASC
                LIMIT 1
                "#,
            )
            .map_err(|e| format!("查询失败: {}", e))?;

        let mut rows = stmt
            .query(params![uid])
            .map_err(|e| format!("查询失败: {}", e))?;

        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let id: i32 = row.get::<_, i64>(0).unwrap_or(0) as i32;
            let type_str: String = row.get(1).unwrap_or_default();
            let start_time: i64 = row.get(2).unwrap_or(0);
            let calories: i32 = row.get::<_, i64>(3).unwrap_or(0) as i32;
            let distance_meter: i32 = row.get::<_, i64>(4).unwrap_or(0) as i32;
            let duration_second: i32 = row.get::<_, i64>(5).unwrap_or(0) as i32;
            let heart_rate_avg: i32 = row.get::<_, i64>(6).unwrap_or(0) as i32;
            let heart_rate_max: i32 = row.get::<_, i64>(7).unwrap_or(0) as i32;
            let pace_average: String = row.get(8).unwrap_or_default();
            let extra_json: String = row.get(9).unwrap_or_else(|_| "{}".to_string());
            let tracks_json: String = row.get(10).unwrap_or_else(|_| "[]".to_string());

            let extra: Swimming = serde_json::from_str(&extra_json).unwrap_or_default();
            let tracks: Vec<Track> = serde_json::from_str(&tracks_json).unwrap_or_default();

            Ok(Some(Sport {
                id,
                r#type: SportType::from_str(&type_str),
                start_time,
                calories,
                distance_meter,
                duration_second,
                heart_rate_avg,
                heart_rate_max,
                pace_average,
                extra,
                tracks,
            }))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl UserDao for SqliteImpl {
    async fn insert(&self, user: User) -> Result<i32, String> {
        let conn = self.get_conn()?;
        conn
            .execute(
                r#"
                INSERT INTO users (name, password, nickname)
                VALUES (?, ?, ?)
                "#,
                params![user.name, user.password, user.nickname],
            )
            .map_err(|e| format!("插入用户失败: {}", e))?;
        Ok(conn.last_insert_rowid() as i32)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<UserInfo>, String> {
        let conn = self.get_conn()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT u.nickname, COALESCE(a.data, u.avatar, '') AS avatar
                FROM users u
                LEFT JOIN avatars a ON a.uid = u.id
                WHERE u.id = ?
                "#,
            )
            .map_err(|e| format!("查询用户失败: {}", e))?;

        let mut rows = stmt
            .query(params![id])
            .map_err(|e| format!("查询用户失败: {}", e))?;

        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let nickname: String = row.get(0).unwrap_or_default();
            let avatar: String = row.get(1).unwrap_or_default();
            Ok(Some(UserInfo { nickname, avatar }))
        } else {
            Ok(None)
        }
    }

    async fn login(&self, name: &str, password: &str) -> Result<Option<User>, String> {
        let conn = self.get_conn()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, name, password, nickname FROM users WHERE name = ? AND password = ?
                "#,
            )
            .map_err(|e| format!("查询用户失败: {}", e))?;

        let mut rows = stmt
            .query(params![name, password])
            .map_err(|e| format!("查询用户失败: {}", e))?;

        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let id: i32 = row.get::<_, i64>(0).unwrap_or(0) as i32;
            let name: String = row.get(1).unwrap_or_default();
            let password: String = row.get(2).unwrap_or_default();
            let nickname: String = row.get(3).unwrap_or_default();
            Ok(Some(User { id, name, password, nickname }))
        } else {
            Ok(None)
        }
    }

    async fn set_avatar(&self, uid: i32, base64: String) -> Result<(), String> {
        let conn = self.get_conn()?;
        conn.execute(
            r#"
            INSERT INTO avatars (uid, data, mime, created_at, updated_at)
            VALUES (?, ?, 'image/jpeg', strftime('%s','now'), strftime('%s','now'))
            ON CONFLICT(uid) DO UPDATE SET data = excluded.data, mime = excluded.mime, updated_at = strftime('%s','now')
            "#,
            params![uid, base64],
        )
        .map_err(|e| format!("更新头像失败: {}", e))?;
        Ok(())
    }

}

// 删除 SqliteUserDao，直接使用 SqliteImpl 作为 UserDao 实现
