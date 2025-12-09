use async_trait::async_trait;
use sea_orm::{Database, DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait, QueryOrder, Set, ActiveModelTrait, ConnectionTrait, Statement, DbBackend, TransactionTrait};
use serde_json;

use crate::dao::entities::{self, avatars, users};
use crate::model::sport::{Sport, SportExtra, Track, SportType, Swimming};
use crate::model::sport_db::{DbSportExtra, DbSportTrack};
use crate::model::user::{User, UserInfo};
use super::idl::{SportDao, UserDao};

pub struct SqliteImpl {
    conn: DatabaseConnection,
}

impl SqliteImpl {
    pub async fn new(database_path: &str) -> Result<Self, String> {
        let url = format!("sqlite://{}?mode=rwc", database_path);
        let conn = Database::connect(url).await.map_err(|e| format!("连接数据库失败: {}", e))?;
        let dao = Self { conn };
        dao.init_schema().await?;
        dao.write_check().await?;
        Ok(dao)
    }

    // 提供同步构造以兼容现有调用点
    pub fn new_sync(database_path: &str) -> Result<Self, String> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建运行时失败: {}", e))?;
        rt.block_on(Self::new(database_path))
    }

    async fn exec_batch(&self, sql: &str) -> Result<(), String> {
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

    async fn init_schema(&self) -> Result<(), String> {
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
        let _ = self.exec_batch("ALTER TABLE users ADD COLUMN nickname TEXT NOT NULL DEFAULT '';").await;
        let _ = self.exec_batch("ALTER TABLE users ADD COLUMN avatar TEXT NOT NULL DEFAULT '';").await;
        Ok(())
    }

    async fn write_check(&self) -> Result<(), String> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS __healthcheck (id INTEGER PRIMARY KEY AUTOINCREMENT, n INTEGER NOT NULL);
        "#;
        self.exec_batch(sql).await?;
        self.exec_batch("INSERT INTO __healthcheck (n) VALUES (1);").await?;
        self.exec_batch("DELETE FROM __healthcheck;").await?;
        Ok(())
    }
}

#[async_trait]
impl SportDao for SqliteImpl {
    async fn insert(&self, uid: i32, sport: Sport) -> Result<(), String> {
        let extra_tagged = sport.extra.clone().map(DbSportExtra::from);
        let extra_json = serde_json::to_string(&extra_tagged)
            .map_err(|e| format!("extra 序列化失败: {}", e))?;
        let db_tracks: Vec<DbSportTrack> = sport.tracks.clone().into_iter().map(DbSportTrack::from).collect();
        let tracks_json = serde_json::to_string(&db_tracks)
            .map_err(|e| format!("tracks 序列化失败: {}", e))?;

        let mut am: entities::ActiveModel = Default::default();
        am.uid = Set(uid);
        am.type_ = Set(sport.r#type.as_str().to_string());
        am.start_time = Set(sport.start_time);
        am.calories = Set(sport.calories);
        am.distance_meter = Set(sport.distance_meter);
        am.duration_second = Set(sport.duration_second);
        am.heart_rate_avg = Set(sport.heart_rate_avg);
        am.heart_rate_max = Set(sport.heart_rate_max);
        am.pace_average = Set(sport.pace_average);
        am.extra = Set(extra_json);
        am.tracks = Set(tracks_json);
        am.insert(&self.conn).await.map_err(|e| format!("插入失败: {}", e))?;
        Ok(())
    }

    async fn insert_many(&self, uid: i32, sports: Vec<Sport>) -> Result<usize, String> {
        let mut count = 0usize;
        self.conn.transaction(|txn| {
            Box::pin(async move {
                for sport in sports {
                    let extra_tagged = sport.extra.clone().map(DbSportExtra::from);
                    let extra_json = serde_json::to_string(&extra_tagged).map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                    let db_tracks: Vec<DbSportTrack> = sport.tracks.clone().into_iter().map(DbSportTrack::from).collect();
                    let tracks_json = serde_json::to_string(&db_tracks).map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                    let mut am: entities::ActiveModel = Default::default();
                    am.uid = Set(uid);
                    am.type_ = Set(sport.r#type.as_str().to_string());
                    am.start_time = Set(sport.start_time);
                    am.calories = Set(sport.calories);
                    am.distance_meter = Set(sport.distance_meter);
                    am.duration_second = Set(sport.duration_second);
                    am.heart_rate_avg = Set(sport.heart_rate_avg);
                    am.heart_rate_max = Set(sport.heart_rate_max);
                    am.pace_average = Set(sport.pace_average);
                    am.extra = Set(extra_json);
                    am.tracks = Set(tracks_json);
                    am.insert(txn).await.map_err(|e| e)?;
                    count += 1;
                }
                Ok::<_, sea_orm::DbErr>(count)
            })
        }).await.map_err(|e| format!("提交事务失败: {}", e))
    }

    async fn list(&self, uid: i32, page: i32, size: i32) -> Result<Vec<Sport>, String> {
        let safe_size = if size <= 0 { 20 } else { size.min(100) } as i64;
        let safe_page = if page < 0 { 0 } else { page } as i64;
        let models = entities::Entity::find()
            .filter(entities::Column::Uid.eq(uid))
            .order_by_desc(entities::Column::StartTime)
            .paginate(&self.conn, safe_size as u64)
            .fetch_page(safe_page as u64)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;
        let result = models
            .into_iter()
            .map(|m| {
                let extra: Option<SportExtra> = parse_extra_compat(&m.extra);
                let tracks: Vec<Track> = parse_tracks_compat(&m.tracks);
                Sport {
                    id: m.id,
                    r#type: SportType::from_str(&m.type_),
                    start_time: m.start_time,
                    calories: m.calories,
                    distance_meter: m.distance_meter,
                    duration_second: m.duration_second,
                    heart_rate_avg: m.heart_rate_avg,
                    heart_rate_max: m.heart_rate_max,
                    pace_average: m.pace_average,
                    extra,
                    tracks,
                }
            })
            .collect();
        Ok(result)
    }

    async fn list_by_time_range(&self, uid: i32, start_time: i64, end_time: i64) -> Result<Vec<Sport>, String> {
        let models = entities::Entity::find()
            .filter(entities::Column::Uid.eq(uid))
            .filter(entities::Column::StartTime.gte(start_time))
            .filter(entities::Column::StartTime.lte(end_time))
            .order_by_desc(entities::Column::StartTime)
            .all(&self.conn)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;
        let result = models
            .into_iter()
            .map(|m| {
                let extra: Option<SportExtra> = parse_extra_compat(&m.extra);
                let tracks: Vec<Track> = parse_tracks_compat(&m.tracks);
                Sport {
                    id: m.id,
                    r#type: SportType::from_str(&m.type_),
                    start_time: m.start_time,
                    calories: m.calories,
                    distance_meter: m.distance_meter,
                    duration_second: m.duration_second,
                    heart_rate_avg: m.heart_rate_avg,
                    heart_rate_max: m.heart_rate_max,
                    pace_average: m.pace_average,
                    extra,
                    tracks,
                }
            })
            .collect();
        Ok(result)
    }

    async fn update(&self, uid: i32, sport: Sport) -> Result<(), String> {
        if sport.id <= 0 { return Err("invalid sport id".to_string()); }
        let extra_tagged = sport.extra.clone().map(DbSportExtra::from);
        let extra_json = serde_json::to_string(&extra_tagged)
            .map_err(|e| format!("extra 序列化失败: {}", e))?;
        let db_tracks: Vec<DbSportTrack> = sport.tracks.clone().into_iter().map(DbSportTrack::from).collect();
        let tracks_json = serde_json::to_string(&db_tracks)
            .map_err(|e| format!("tracks 序列化失败: {}", e))?;
        let model = entities::Entity::find_by_id(sport.id)
            .one(&self.conn)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;
        let Some(model) = model else { return Err("记录不存在".to_string()); };
        if model.uid != uid { return Err("记录不存在或无权限".to_string()); }
        let mut am: entities::ActiveModel = model.into();
        am.type_ = Set(sport.r#type.as_str().to_string());
        am.start_time = Set(sport.start_time);
        am.calories = Set(sport.calories);
        am.distance_meter = Set(sport.distance_meter);
        am.duration_second = Set(sport.duration_second);
        am.heart_rate_avg = Set(sport.heart_rate_avg);
        am.heart_rate_max = Set(sport.heart_rate_max);
        am.pace_average = Set(sport.pace_average);
        am.extra = Set(extra_json);
        am.tracks = Set(tracks_json);
        am.update(&self.conn).await.map_err(|e| format!("更新失败: {}", e))?;
        Ok(())
    }

    async fn remove(&self, uid: i32, id: i32) -> Result<(), String> {
        if id <= 0 { return Err("invalid sport id".to_string()); }
        let res = entities::Entity::delete_many()
            .filter(entities::Column::Id.eq(id))
            .filter(entities::Column::Uid.eq(uid))
            .exec(&self.conn)
            .await
            .map_err(|e| format!("删除失败: {}", e))?;
        if res.rows_affected == 0 { return Err("记录不存在或无权限".to_string()); }
        Ok(())
    }

    async fn get_by_id(&self, uid: i32, id: i32) -> Result<Option<Sport>, String> {
        if id <= 0 { return Ok(None); }
        let model = entities::Entity::find()
            .filter(entities::Column::Id.eq(id))
            .filter(entities::Column::Uid.eq(uid))
            .one(&self.conn)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;
        Ok(model.map(|m| {
            let extra: Option<SportExtra> = parse_extra_compat(&m.extra);
            let tracks: Vec<Track> = parse_tracks_compat(&m.tracks);
            Sport {
                id: m.id,
                r#type: SportType::from_str(&m.type_),
                start_time: m.start_time,
                calories: m.calories,
                distance_meter: m.distance_meter,
                duration_second: m.duration_second,
                heart_rate_avg: m.heart_rate_avg,
                heart_rate_max: m.heart_rate_max,
                pace_average: m.pace_average,
                extra,
                tracks,
            }
        }))
    }

    async fn get_first(&self, uid: i32) -> Result<Option<Sport>, String> {
        let model = entities::Entity::find()
            .filter(entities::Column::Uid.eq(uid))
            .order_by_asc(entities::Column::StartTime)
            .one(&self.conn)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;
        Ok(model.map(|m| {
            let extra: Option<SportExtra> = parse_extra_compat(&m.extra);
            let tracks: Vec<Track> = parse_tracks_compat(&m.tracks);
            Sport {
                id: m.id,
                r#type: SportType::from_str(&m.type_),
                start_time: m.start_time,
                calories: m.calories,
                distance_meter: m.distance_meter,
                duration_second: m.duration_second,
                heart_rate_avg: m.heart_rate_avg,
                heart_rate_max: m.heart_rate_max,
                pace_average: m.pace_average,
                extra,
                tracks,
            }
        }))
    }
}

fn parse_extra_compat(extra_json: &str) -> Option<SportExtra> {
    if extra_json.trim().is_empty() { return None; }
    serde_json::from_str::<Option<DbSportExtra>>(extra_json)
        .map(|o| o.map(SportExtra::from))
        .or_else(|_| serde_json::from_str::<Swimming>(extra_json).map(|s| Some(SportExtra::Swimming(s))))
        .ok()
        .flatten()
}

fn parse_tracks_compat(tracks_json: &str) -> Vec<Track> {
    if tracks_json.trim().is_empty() { return Vec::new(); }
    serde_json::from_str::<Vec<DbSportTrack>>(tracks_json)
        .map(|v| v.into_iter().map(Track::from).collect())
        .or_else(|_| serde_json::from_str::<Vec<Track>>(tracks_json))
        .unwrap_or_default()
}

#[async_trait]
impl UserDao for SqliteImpl {
    async fn insert(&self, user: User) -> Result<i32, String> {
        let mut am: users::ActiveModel = Default::default();
        am.name = Set(user.name);
        am.password = Set(user.password);
        am.nickname = Set(user.nickname);
        am.avatar = Set(String::new());
        let res = users::Entity::insert(am)
            .exec(&self.conn)
            .await
            .map_err(|e| format!("插入用户失败: {}", e))?;
        Ok(res.last_insert_id)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<UserInfo>, String> {
        let user = users::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(|e| format!("查询用户失败: {}", e))?;
        let Some(user) = user else { return Ok(None); };
        let avatar = avatars::Entity::find()
            .filter(avatars::Column::Uid.eq(id))
            .one(&self.conn)
            .await
            .map_err(|e| format!("查询头像失败: {}", e))?;
        let avatar_data = avatar.map(|a| a.data).unwrap_or_else(|| user.avatar);
        Ok(Some(UserInfo { nickname: user.nickname, avatar: avatar_data }))
    }

    async fn login(&self, name: &str, password: &str) -> Result<Option<User>, String> {
        let user = users::Entity::find()
            .filter(users::Column::Name.eq(name.to_string()))
            .filter(users::Column::Password.eq(password.to_string()))
            .one(&self.conn)
            .await
            .map_err(|e| format!("查询用户失败: {}", e))?;
        Ok(user.map(|u| User { id: u.id, name: u.name, password: u.password, nickname: u.nickname }))
    }

    async fn set_avatar(&self, uid: i32, base64: String) -> Result<(), String> {
        // 使用原生 SQL 实现 SQLite 的 UPSERT 逻辑
        let sql = format!(
            "INSERT INTO avatars (uid, data, mime, created_at, updated_at)\n             VALUES ({}, '{}', 'image/jpeg', strftime('%s','now'), strftime('%s','now'))\n             ON CONFLICT(uid) DO UPDATE SET data = excluded.data, mime = excluded.mime, updated_at = strftime('%s','now')",
            uid,
            base64.replace("'", "''"),
        );
        self.conn
            .execute(Statement::from_string(DbBackend::Sqlite, sql))
            .await
            .map_err(|e| format!("更新头像失败: {}", e))?;
        Ok(())
    }
}
