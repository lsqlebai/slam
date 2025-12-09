use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc, Weekday};
use ctx_marco::inject_ctx;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::dao::idl::SportDao;
use crate::dao::cache::ResultCache;
use crate::handlers::jwt::Context;
use crate::model::sport::{Sport, SportType, SportExtra};
use crate::service::common::ServiceError;

pub struct SportService {
    dao: Arc<dyn SportDao + Send + Sync>,
    cache_total: Arc<dyn ResultCache<StatSummary, i32> + Send + Sync>,
    cache_year: Arc<dyn ResultCache<StatSummary, String> + Send + Sync>,
}

impl SportService {
    pub fn new(
        dao: Arc<dyn SportDao + Send + Sync>, 
        cache_total: Arc<dyn ResultCache<StatSummary, i32> + Send + Sync>,
        cache_year: Arc<dyn ResultCache<StatSummary, String> + Send + Sync>,
    ) -> Self {
        Self { dao, cache_total, cache_year }
    }

    #[inject_ctx]
    pub async fn insert(&self, sport: Sport) -> Result<(), ServiceError> {
        let y = DateTime::from_timestamp(sport.start_time, 0).map(|dt| dt.year());
        self
            .dao
            .insert(ctx.uid, sport)
            .await
            .map_err(|e| ServiceError { code: 500, message: e })?;
        self.cache_total.invalidate(ctx.uid).await;
        if let Some(year) = y { 
            let key = format!("{}@{}", ctx.uid, year);
            self.cache_year.invalidate(key).await;
        }
        Ok(())
    }

    #[inject_ctx]
    pub async fn list(&self, page: i32, size: i32) -> Result<Vec<Sport>, ServiceError> {
        self.dao
            .list(ctx.uid, page, size)
            .await
            .map_err(|e| ServiceError {
                code: 500,
                message: e,
            })
    }
    #[inject_ctx]
    pub async fn update(&self, sport: Sport) -> Result<(), ServiceError> {
        let old = self.dao.get_by_id(ctx.uid, sport.id).await.map_err(|e| ServiceError { code: 500, message: e })?;
        let ny = DateTime::from_timestamp(sport.start_time, 0).map(|dt| dt.year());
        self
            .dao
            .update(ctx.uid, sport)
            .await
            .map_err(|e| ServiceError { code: 500, message: e })?;
        self.cache_total.invalidate(ctx.uid).await;
        if let Some(o) = old { 
            let oy = DateTime::from_timestamp(o.start_time, 0).map(|dt| dt.year());
            if let Some(oyr) = oy { 
                let key = format!("{}@{}", ctx.uid, oyr);
                self.cache_year.invalidate(key).await;
            }
        }
        if let Some(nyr) = ny { 
            let key = format!("{}@{}", ctx.uid, nyr);
            self.cache_year.invalidate(key).await;
        }
        Ok(())
    }

    #[inject_ctx]
    pub async fn import<R: std::io::Read>(
        &self,
        vendor: String,
        reader: csv::Reader<R>,
    ) -> Result<usize, ServiceError> {
        let mut r = reader;
        let sports = parse_sports_from_csv(&vendor, &mut r);
        if sports.is_empty() {
            return Err(ServiceError {
                code: 400,
                message: "no valid rows".to_string(),
            });
        }
        // 校验类型一致性，发现不一致直接报错（避免错误数据入库）
        for (i, s) in sports.iter().enumerate() {
            if let Err(e) = s.validate_type_consistency() {
                return Err(ServiceError { code: 400, message: format!("row {}: {}", i, e) });
            }
        }
        let mut years: std::collections::HashSet<i32> = std::collections::HashSet::new();
        for s in &sports { 
            if let Some(y) = DateTime::from_timestamp(s.start_time, 0).map(|dt| dt.year()) { years.insert(y); }
        }
        let inserted = self
            .dao
            .insert_many(ctx.uid, sports)
            .await
            .map_err(|e| ServiceError { code: 500, message: e })?;
        self.cache_total.invalidate(ctx.uid).await;
        for y in years { 
            let key = format!("{}@{}", ctx.uid, y);
            self.cache_year.invalidate(key).await;
        }
        Ok(inserted)
    }

    #[inject_ctx]
    pub async fn delete(&self, id: i32) -> Result<(), ServiceError> {
        let old = self.dao.get_by_id(ctx.uid, id).await.map_err(|e| ServiceError { code: 500, message: e })?;
        self
            .dao
            .remove(ctx.uid, id)
            .await
            .map_err(|e| ServiceError { code: 500, message: e })?;
        self.cache_total.invalidate(ctx.uid).await;
        if let Some(o) = old { 
            if let Some(y) = DateTime::from_timestamp(o.start_time, 0).map(|dt| dt.year()) { 
                let key = format!("{}@{}", ctx.uid, y);
                self.cache_year.invalidate(key).await;
            }
        }
        Ok(())
    }

    #[inject_ctx]
    pub async fn stats(&self, spec: StatsParam) -> Result<StatSummary, ServiceError> {
        if let StatKind::Total = spec.kind {
            if let Some(cached) = self.cache_total.get(ctx.uid).await {
                return Ok(cached);
            }
        }
        if let StatKind::Year = spec.kind {
            let key = format!("{}@{}", ctx.uid, spec.year);
            if let Some(cached) = self.cache_year.get(key.clone()).await {
                return Ok(cached);
            }
        }
        let (start_time, end_time) = match spec.kind {
            StatKind::Year => {
                let y = spec.year;
                let start = Utc.with_ymd_and_hms(y, 1, 1, 0, 0, 0).unwrap().timestamp();
                let end = Utc
                    .with_ymd_and_hms(y + 1, 1, 1, 0, 0, 0)
                    .unwrap()
                    .timestamp();
                (start, end)
            }
            StatKind::Month => {
                let y = spec.year;
                let m = spec.month.ok_or(ServiceError {
                    code: 400,
                    message: "invalid month".to_string(),
                })?;
                let start = Utc.with_ymd_and_hms(y, m, 1, 0, 0, 0).unwrap().timestamp();
                let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
                let end = Utc
                    .with_ymd_and_hms(ny, nm, 1, 0, 0, 0)
                    .unwrap()
                    .timestamp();
                (start, end)
            }
            StatKind::Week => {
                let y = spec.year;
                let w = spec.week.ok_or(ServiceError {
                    code: 400,
                    message: "invalid week".to_string(),
                })?;
                let start_date =
                    NaiveDate::from_isoywd_opt(y, w, Weekday::Mon).ok_or(ServiceError {
                        code: 400,
                        message: "invalid iso week".to_string(),
                    })?;
                let start_dt = Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap());
                let end_dt = start_dt + Duration::days(7);
                (start_dt.timestamp(), end_dt.timestamp())
            }
            StatKind::Total => (0, i64::MAX),
        };
        let sports = self
            .dao
            .list_by_time_range(ctx.uid, start_time, end_time)
            .await
            .map_err(|e| ServiceError {
                code: 500,
                message: e,
            })?;
        let total_count: i32 = sports.len() as i32;
        let total_calories: i32 = sports.iter().map(|s| s.calories).sum();
        let total_duration_second: i32 = sports.iter().map(|s| s.duration_second).sum();
        let total_distance_meter: i32 = sports.iter().map(|s| s.distance_meter).sum();
        let buckets = match spec.kind {
            StatKind::Year => group_by_month(sports.clone()),
            StatKind::Month => group_by_month_day(sports.clone()),
            StatKind::Week => group_by_week_day(sports.clone()),
            StatKind::Total => Vec::new(),
        };
        let earliest_year = match spec.kind {
            StatKind::Year => match self.dao.get_first(ctx.uid).await {
                Ok(Some(first)) => {
                    DateTime::from_timestamp(first.start_time, 0).map(|dt| dt.year())
                }
                Ok(None) => None,
                Err(_) => None,
            },
            _ => None,
        };
        let type_buckets = group_by_type(sports.clone());
        let sports_field = match spec.kind {
            StatKind::Total => Vec::new(),
            _ => sports,
        };
        let summary = StatSummary {
            buckets,
            type_buckets,
            total_count,
            total_calories,
            total_duration_second,
            total_distance_meter,
            sports: sports_field,
            earliest_year,
        };
        if let StatKind::Total = spec.kind {
            self.cache_total.set(ctx.uid, summary.clone()).await;
        }
        if let StatKind::Year = spec.kind {
            let key = format!("{}@{}", ctx.uid, spec.year);
            self.cache_year.set(key.clone(), summary.clone()).await;
        }
        Ok(summary)
    }

    pub async fn group_by_year(
        &self,
        uid: i32,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<StatBucket>, ServiceError> {
        let items = self
            .dao
            .list_by_time_range(uid, start_time, end_time)
            .await
            .map_err(|e| ServiceError {
                code: 500,
                message: e,
            })?;
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
pub enum StatKind {
    Year,
    Month,
    Week,
    Total,
}

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
    pub type_buckets: Vec<TypeBucket>,
    pub total_count: i32,
    pub total_calories: i32,
    pub total_duration_second: i32,
    pub total_distance_meter: i32,
    pub sports: Vec<Sport>,
    pub earliest_year: Option<i32>,
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
    let mut acc: std::collections::HashMap<u32, StatBucket> = std::collections::HashMap::new();
    for sport in items.into_iter() {
        let dt = DateTime::from_timestamp(sport.start_time, 0).expect("invalid timestamp");
        let k = key(&dt);
        let entry = acc.entry(k).or_insert(StatBucket {
            date: k as i32,
            duration: 0,
            calories: 0,
            count: 0,
        });
        entry.count += 1;
        entry.duration += sport.duration_second;
        entry.calories += sport.calories;
    }
    let mut v: Vec<StatBucket> = acc.into_values().collect();
    v.sort_by_key(|b| b.date);
    v
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct TypeBucket {
    pub r#type: SportType,
    pub duration: i32,
    pub calories: i32,
    pub count: i32,
    pub distance_meter: i32,
}

fn group_by_type(items: Vec<Sport>) -> Vec<TypeBucket> {
    let mut acc: std::collections::HashMap<SportType, TypeBucket> =
        std::collections::HashMap::new();
    for sport in items.into_iter() {
        let key = sport.r#type;
        let entry = acc.entry(key).or_insert(TypeBucket {
            r#type: key,
            duration: 0,
            calories: 0,
            count: 0,
            distance_meter: 0,
        });
        entry.count += 1;
        entry.duration += sport.duration_second;
        entry.calories += sport.calories;
        entry.distance_meter += sport.distance_meter;
    }
    let mut v: Vec<TypeBucket> = acc.into_values().collect();
    v.sort_by_key(|b| b.r#type.as_str().to_string());
    v
}

trait VendorFileParser {
    fn parse<R: std::io::Read>(&self, reader: &mut csv::Reader<R>) -> Vec<Sport>;
}

struct HuaweiParser;
struct XiaomiParser;

impl VendorFileParser for HuaweiParser {
    fn parse<R: std::io::Read>(&self, _reader: &mut csv::Reader<R>) -> Vec<Sport> {
        panic!("huawei parser not implemented")
    }
}

#[derive(Deserialize)]
struct XiaomiCsvRow {
    #[serde(rename = "Time")]
    time: i64,
    #[serde(rename = "Category")]
    category: String,
    #[serde(rename = "Value")]
    value: String,
}

#[derive(Deserialize, Default)]
struct XiaomiValue {
    #[serde(rename = "calories")]
    calories: Option<i32>,
    #[serde(rename = "total_cal")]
    total_cal: Option<i32>,
    #[serde(rename = "distance")]
    distance: Option<i32>,
    #[serde(rename = "duration")]
    duration: Option<i32>,
    #[serde(rename = "valid_duration")]
    valid_duration: Option<i32>,
    #[serde(rename = "avg_swolf")]
    avg_swolf: Option<i32>,
    //#[serde(rename = "best_swolf")] best_swolf: Option<i32>, // 未被使用，保留供后续扩展
    #[serde(rename = "main_posture")]
    main_posture: Option<i32>,
    #[serde(rename = "max_stroke_freq")]
    max_stroke_freq: Option<i32>,
    //#[serde(rename = "stroke_count")] stroke_count: Option<i32>,// 未被使用，保留供后续扩展
    //#[serde(rename = "turn_count")] turn_count: Option<i32>,// 未被使用，保留供后续扩展
    //#[serde(rename = "pool_width")] pool_width: Option<i32>,// 未被使用，保留供后续扩展
    //#[serde(rename = "start_time")] start_time: Option<i64>,// 未被使用，保留供后续扩展
    //#[serde(rename = "time")] value_time: Option<i64>,// 未被使用，保留供后续扩展
    //#[serde(rename = "end_time")] end_time: Option<i64>,// 未被使用，保留供后续扩展
}

impl VendorFileParser for XiaomiParser {
    fn parse<R: std::io::Read>(&self, reader: &mut csv::Reader<R>) -> Vec<Sport> {
        let mut res = Vec::new();
        for rec in reader.deserialize() {
            let Ok(row) = rec else { continue };
            let row: XiaomiCsvRow = row;
            let mut start_time = row.time;
            if start_time > 1_000_000_000_000 {
                start_time /= 1000;
            }
            if row.category.to_lowercase() != "swimming" {
                continue;
            }
            let parsed: XiaomiValue = serde_json::from_str(&row.value).unwrap_or_default();
            let calories = parsed.calories.or(parsed.total_cal).unwrap_or(0);
            let distance_meter = parsed.distance.unwrap_or(0);
            let duration_second = parsed.valid_duration.or(parsed.duration).unwrap_or(0);
            let swolf_avg = parsed.avg_swolf.unwrap_or(0);
            let stroke_avg = parsed.max_stroke_freq.unwrap_or(0);
            let main_stroke = match parsed.main_posture.unwrap_or(0) {
                1 => "freestyle",
                2 => "backstroke",
                3 => "breaststroke",
                4 => "butterfly",
                _ => "unknown",
            };
            let sport = Sport {
                id: 0,
                r#type: SportType::Swimming,
                start_time,
                calories,
                distance_meter,
                duration_second,
                heart_rate_avg: 0,
                heart_rate_max: 0,
                pace_average: String::new(),
                extra: Some(SportExtra::Swimming(crate::model::sport::Swimming { 
                    main_stroke: main_stroke.to_string(),
                    stroke_avg,
                    swolf_avg,
                })),
                tracks: vec![],
            };
            res.push(sport);
        }
        res
    }
}

// removed unused common parser placeholder

pub fn parse_sports_from_csv<R: std::io::Read>(
    vendor: &str,
    reader: &mut csv::Reader<R>,
) -> Vec<Sport> {
    match vendor.to_lowercase().as_str() {
        "huawei" => HuaweiParser.parse(reader),
        "xiaomi" => XiaomiParser.parse(reader),
        _ => panic!("unsupported vendor"),
    }
}
