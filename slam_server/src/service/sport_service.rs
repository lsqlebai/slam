use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc, Weekday};

use crate::dao::idl::SportDao;
use crate::model::sport::Sport;
use crate::service::common::{ServiceError};

pub struct SportService {
    dao: Arc<dyn SportDao + Send + Sync>,
}

impl SportService {
    pub fn new(dao: Arc<dyn SportDao + Send + Sync>) -> Self { Self { dao } }

    pub async fn insert(&self, uid: i32, sport: Sport) -> Result<(), ServiceError> {
        self.dao.insert(uid, sport).await.map_err(|e| ServiceError { code: 500, message: e })
    }

    pub async fn list(&self, uid: i32, page: i32, size: i32) -> Result<Vec<Sport>, ServiceError> {
        self.dao.list(uid, page, size).await.map_err(|e| ServiceError { code: 500, message: e })
    }

    pub async fn stats(&self, uid: i32, spec: StatsParam) -> Result<StatSummary, ServiceError> {
        let (start_time, end_time) = match spec.kind {
            StatKind::Year => {
                let y = spec.year;
                let start = Utc.with_ymd_and_hms(y, 1, 1, 0, 0, 0).unwrap().timestamp();
                let end = Utc.with_ymd_and_hms(y + 1, 1, 1, 0, 0, 0).unwrap().timestamp();
                (start, end)
            }
            StatKind::Month => {
                let y = spec.year;
                let m = spec.month.ok_or(ServiceError { code: 400, message: "invalid month".to_string() })?;
                let start = Utc.with_ymd_and_hms(y, m, 1, 0, 0, 0).unwrap().timestamp();
                let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
                let end = Utc.with_ymd_and_hms(ny, nm, 1, 0, 0, 0).unwrap().timestamp();
                (start, end)
            }
            StatKind::Week => {
                let y = spec.year;
                let w = spec.week.ok_or(ServiceError { code: 400, message: "invalid week".to_string() })?;
                let start_date = NaiveDate::from_isoywd_opt(y, w, Weekday::Mon).ok_or(ServiceError { code: 400, message: "invalid iso week".to_string() })?;
                let start_dt = Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap());
                let end_dt = start_dt + Duration::days(7);
                (start_dt.timestamp(), end_dt.timestamp())
            }
        };
        let sports = self.dao.list_by_time_range(uid, start_time, end_time).await.map_err(|e| ServiceError { code: 500, message: e })?;
        let total_count:i32 = sports.len() as i32;
        let total_calories:i32 = sports.iter().map(|s| s.calories).sum();
        let total_duration_second:i32 = sports.iter().map(|s| s.duration_second).sum();
        let buckets = match spec.kind {
            StatKind::Year => group_by_month(sports.clone()),
            StatKind::Month => group_by_month_day(sports.clone()),
            StatKind::Week => group_by_week_day(sports.clone()),
        };
        Ok(StatSummary { buckets, total_count, total_calories, total_duration_second, sports })
    }

    pub async fn group_by_year(&self, uid: i32, start_time: i64, end_time: i64) -> Result<Vec<StatBucket>, ServiceError> {
        let items = self
            .dao
            .list_by_time_range(uid, start_time, end_time)
            .await
            .map_err(|e| ServiceError { code: 500, message: e })?;
        Ok(group_by_month(items))
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct StatBucket {
    pub date: i32,
    pub duration: i32,
    pub calories: i32,
    pub count: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum StatKind { Year, Month, Week }

#[derive(Debug, Clone, Copy)]
pub struct StatsParam {
    pub kind: StatKind,
    pub year: i32,
    pub month: Option<u32>,
    pub week: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct StatSummary {
    pub buckets: Vec<StatBucket>,
    pub total_count: i32,
    pub total_calories: i32,
    pub total_duration_second: i32,
    pub sports: Vec<Sport>,
}

fn group_by_month(items: Vec<Sport>) -> Vec<StatBucket> {
    group_by_key(items, |dt| dt.month())
}

fn group_by_month_day(items: Vec<Sport>) -> Vec<StatBucket> {
    group_by_key(items, |dt| dt.day())
}

fn group_by_week_day(items: Vec<Sport>) -> Vec<StatBucket> {
    group_by_key(items, |dt| dt.weekday().num_days_from_monday() + 1)
}

fn group_by_key(items: Vec<Sport>, key: impl Fn(&DateTime<Utc>) -> u32) -> Vec<StatBucket> {
    let mut acc: HashMap<u32, StatBucket> = HashMap::new();
    for sport in items.into_iter() {
        let dt = DateTime::from_timestamp(sport.start_time, 0).expect("invalid timestamp");
        let k = key(&dt);
        let entry = acc.entry(k).or_insert(StatBucket { date: k as i32, duration: 0, calories: 0, count: 0 });
        entry.count += 1;
        entry.duration += sport.duration_second;
        entry.calories += sport.calories;
    }
    let mut v: Vec<StatBucket> = acc.into_iter().map(|(_, b)| b).collect();
    v.sort_by_key(|b| b.date);
    v
}