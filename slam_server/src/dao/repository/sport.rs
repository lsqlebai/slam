use super::Repository;
use super::compat::{parse_extra_compat, parse_tracks_compat};
use crate::dao::entities::{self};
use crate::dao::entities::{DbSportExtra, DbSportTrack};
use crate::dao::idl::SportDao;
use crate::model::ai_job::{AiJobSubmission, JOB_READY, JOB_SUBMITTED};
use crate::model::sport::{Sport, SportExtra, SportType, Track};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set, Statement, TransactionTrait,
};

#[async_trait]
impl SportDao for Repository {
    async fn insert(&self, uid: i32, sport: Sport) -> Result<(), String> {
        let extra_tagged = sport.extra.clone().map(DbSportExtra::from);
        let extra_json =
            serde_json::to_string(&extra_tagged).map_err(|e| format!("extra 序列化失败: {}", e))?;
        let db_tracks: Vec<DbSportTrack> = sport
            .tracks
            .clone()
            .into_iter()
            .map(DbSportTrack::from)
            .collect();
        let tracks_json =
            serde_json::to_string(&db_tracks).map_err(|e| format!("tracks 序列化失败: {}", e))?;

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
        am.insert(&self.conn)
            .await
            .map_err(|e| format!("插入失败: {}", e))?;
        Ok(())
    }

    async fn insert_many(&self, uid: i32, sports: Vec<Sport>) -> Result<usize, String> {
        let mut count = 0usize;
        self.conn
            .transaction(|txn| {
                Box::pin(async move {
                    for sport in sports {
                        let extra_tagged = sport.extra.clone().map(DbSportExtra::from);
                        let extra_json = serde_json::to_string(&extra_tagged)
                            .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                        let db_tracks: Vec<DbSportTrack> = sport
                            .tracks
                            .clone()
                            .into_iter()
                            .map(DbSportTrack::from)
                            .collect();
                        let tracks_json = serde_json::to_string(&db_tracks)
                            .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
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
            })
            .await
            .map_err(|e| format!("提交事务失败: {}", e))
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

    async fn list_by_time_range(
        &self,
        uid: i32,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<Sport>, String> {
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
        if sport.id <= 0 {
            return Err("invalid sport id".to_string());
        }
        let extra_tagged = sport.extra.clone().map(DbSportExtra::from);
        let extra_json =
            serde_json::to_string(&extra_tagged).map_err(|e| format!("extra 序列化失败: {}", e))?;
        let db_tracks: Vec<DbSportTrack> = sport
            .tracks
            .clone()
            .into_iter()
            .map(DbSportTrack::from)
            .collect();
        let tracks_json =
            serde_json::to_string(&db_tracks).map_err(|e| format!("tracks 序列化失败: {}", e))?;
        let model = entities::Entity::find_by_id(sport.id)
            .one(&self.conn)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;
        let Some(model) = model else {
            return Err("记录不存在".to_string());
        };
        if model.uid != uid {
            return Err("记录不存在或无权限".to_string());
        }
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
        am.update(&self.conn)
            .await
            .map_err(|e| format!("更新失败: {}", e))?;
        Ok(())
    }

    async fn remove(&self, uid: i32, id: i32) -> Result<(), String> {
        if id <= 0 {
            return Err("invalid sport id".to_string());
        }
        let res = entities::Entity::delete_many()
            .filter(entities::Column::Id.eq(id))
            .filter(entities::Column::Uid.eq(uid))
            .exec(&self.conn)
            .await
            .map_err(|e| format!("删除失败: {}", e))?;
        if res.rows_affected == 0 {
            return Err("记录不存在或无权限".to_string());
        }
        Ok(())
    }

    async fn get_by_id(&self, uid: i32, id: i32) -> Result<Option<Sport>, String> {
        if id <= 0 {
            return Ok(None);
        }
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

    async fn insert_from_ai_job(
        &self,
        uid: i32,
        sport: Sport,
        job_id: &str,
    ) -> Result<AiJobSubmission, String> {
        let job_id = job_id.to_string();
        self.conn
            .transaction(|txn| {
                Box::pin(async move {
                    let job = txn
                        .query_one(Statement::from_sql_and_values(
                            DbBackend::Sqlite,
                            "SELECT status, submitted_sport_id FROM ai_jobs WHERE uid = ? AND id = ?",
                            vec![uid.into(), job_id.clone().into()],
                        ))
                        .await?
                        .ok_or_else(|| sea_orm::DbErr::Custom("AI任务不存在".to_string()))?;
                    let status: String = job.try_get("", "status")?;
                    let submitted_sport_id: Option<i32> =
                        job.try_get("", "submitted_sport_id")?;
                    if status == JOB_SUBMITTED {
                        let sport_id = submitted_sport_id.ok_or_else(|| {
                            sea_orm::DbErr::Custom("AI任务提交状态异常".to_string())
                        })?;
                        return Ok(AiJobSubmission {
                            sport_id,
                            asset_paths: Vec::new(),
                        });
                    }
                    if status != JOB_READY {
                        return Err(sea_orm::DbErr::Custom(
                            "AI任务尚未识别完成，不能提交".to_string(),
                        ));
                    }

                    let extra_tagged = sport.extra.clone().map(DbSportExtra::from);
                    let extra_json = serde_json::to_string(&extra_tagged)
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                    let db_tracks: Vec<DbSportTrack> = sport
                        .tracks
                        .clone()
                        .into_iter()
                        .map(DbSportTrack::from)
                        .collect();
                    let tracks_json = serde_json::to_string(&db_tracks)
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
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
                    let inserted = am.insert(txn).await?;
                    let now = chrono::Utc::now().timestamp();
                    let updated = txn
                        .execute(Statement::from_sql_and_values(
                            DbBackend::Sqlite,
                            "UPDATE ai_jobs SET status = ?, submitted_sport_id = ?, submitted_at = ?, updated_at = ? WHERE uid = ? AND id = ? AND status = ?",
                            vec![
                                JOB_SUBMITTED.into(),
                                inserted.id.into(),
                                now.into(),
                                now.into(),
                                uid.into(),
                                job_id.clone().into(),
                                JOB_READY.into(),
                            ],
                        ))
                        .await?;
                    if updated.rows_affected() != 1 {
                        return Err(sea_orm::DbErr::Custom(
                            "AI任务状态已变化，请刷新后重试".to_string(),
                        ));
                    }
                    let paths = txn
                        .query_all(Statement::from_sql_and_values(
                            DbBackend::Sqlite,
                            "SELECT original_path, thumbnail_path FROM ai_job_assets WHERE uid = ? AND job_id = ? AND deleted_at IS NULL",
                            vec![uid.into(), job_id.into()],
                        ))
                        .await?;
                    let mut asset_paths = Vec::new();
                    for row in paths {
                        asset_paths.push(row.try_get("", "original_path")?);
                        asset_paths.push(row.try_get("", "thumbnail_path")?);
                    }
                    Ok(AiJobSubmission {
                        sport_id: inserted.id,
                        asset_paths,
                    })
                })
            })
            .await
            .map_err(|e| format!("提交AI识别结果失败: {e}"))
    }
}
