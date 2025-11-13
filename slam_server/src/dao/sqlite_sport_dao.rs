use async_trait::async_trait;
use rusqlite::{params, Connection, OpenFlags};
use serde_json;

use crate::model::sport::{Sport, Swimming, Track};
use super::sport_dao::SportDao;

pub struct SqliteSportDao {
    conn: Connection,
}

impl SqliteSportDao {
    pub async fn new(database_path: &str) -> Result<Self, String> {
        let conn = Connection::open_with_flags(
            database_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .map_err(|e| format!("连接数据库失败: {}", e))?;
        let dao = Self { conn };
        dao.init_schema().await?;
        Ok(dao)
    }

    async fn init_schema(&self) -> Result<(), String> {
        let create_sql = r#"
        CREATE TABLE IF NOT EXISTS sports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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
        "#;
        self.conn
            .execute_batch(create_sql)
            .map_err(|e| format!("建表失败: {}", e))?;
        Ok(())
    }
}

#[async_trait(?Send)]
impl SportDao for SqliteSportDao {
    async fn insert(&self, sport: Sport) -> Result<(), String> {
        let extra_json = serde_json::to_string(&sport.extra)
            .map_err(|e| format!("extra 序列化失败: {}", e))?;
        let tracks_json = serde_json::to_string(&sport.tracks)
            .map_err(|e| format!("tracks 序列化失败: {}", e))?;

        self.conn
            .execute(
                r#"
                INSERT INTO sports (
                    type, start_time, calories, distance_meter, duration_second,
                    heart_rate_avg, heart_rate_max, pace_average, extra, tracks
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    sport.r#type,
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

    async fn list(&self) -> Result<Vec<Sport>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT id, type, start_time, calories, distance_meter, duration_second,
                       heart_rate_avg, heart_rate_max, pace_average, extra, tracks
                FROM sports
                ORDER BY start_time DESC
                "#,
            )
            .map_err(|e| format!("查询失败: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                let id: i32 = row.get::<_, i64>(0).unwrap_or(0) as i32;
                let r#type: String = row.get(1).unwrap_or_default();
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
                    r#type,
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

    async fn list_by_time_range(&self, start_time: i64, end_time: i64) -> Result<Vec<Sport>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT id, type, start_time, calories, distance_meter, duration_second,
                       heart_rate_avg, heart_rate_max, pace_average, extra, tracks
                FROM sports
                WHERE start_time BETWEEN ? AND ?
                ORDER BY start_time DESC
                "#,
            )
            .map_err(|e| format!("查询失败: {}", e))?;

        let rows = stmt
            .query_map(params![start_time, end_time], |row| {
                let id: i32 = row.get::<_, i64>(0).unwrap_or(0) as i32;
                let r#type: String = row.get(1).unwrap_or_default();
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
                    r#type,
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
}
