use quick_xml::de as xml_de;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{NaiveDate, NaiveDateTime, Local, TimeZone};

use super::sport_xml::{SportXML, XMLSportTrack, XMLSportExtra};

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
#[serde(default, rename = "sport")]
pub struct Sport {
    pub id: i32,
    pub r#type: SportType,
    pub start_time: i64,
    pub calories: i32,
    pub distance_meter: i32,
    pub duration_second: i32,
    pub heart_rate_avg: i32,
    pub heart_rate_max: i32,
    pub pace_average: String,
    pub extra: Option<SportExtra>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
#[serde(default)]
pub struct Track {
    pub distance_meter: i32,
    pub duration_second: i32,
    pub pace_average: String,
    pub extra: Option<SportExtra>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
#[serde(default)]
pub struct Swimming {
    pub main_stroke: String,
    pub stroke_avg: i32,
    pub swolf_avg: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
pub struct Running {
    pub speed_avg: f32,
    pub cadence_avg: i32,
    pub stride_length_avg: i32,
    pub steps_total: i32,
    pub pace_min: String,
    pub pace_max: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(untagged)]
pub enum SportExtra {
    Swimming(Swimming),
    Running(Running),
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum SportType {
    #[default]
    Unknown,
    Swimming,
    Running,
    Cycling,
}

impl SportType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SportType::Unknown => "Unknown",
            SportType::Swimming => "Swimming",
            SportType::Running => "Running",
            SportType::Cycling => "Cycling",
        }
    }
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "swimming" => SportType::Swimming,
            "running" => SportType::Running,
            "cycling" => SportType::Cycling,
            _ => SportType::Unknown,
        }
    }
}

impl SportExtra {
    pub fn from_raw(r#type: SportType, raw: XMLSportExtra) -> Option<SportExtra> {
        match r#type {
            SportType::Swimming => {
                let main_stroke = raw.main_stroke.unwrap_or_default();
                let stroke_avg = raw.stroke_avg.unwrap_or(0);
                let swolf_avg = raw.swolf_avg.unwrap_or(0);
                Some(SportExtra::Swimming(Swimming { main_stroke, stroke_avg, swolf_avg }))
            }
            SportType::Running => {
                let speed_avg = raw.speed_avg.unwrap_or(0.0);
                let cadence_avg = raw.cadence_avg.unwrap_or(0);
                let stride_length_avg = raw.stride_length_avg.unwrap_or(0);
                let steps_total = raw.steps_total.unwrap_or(0);
                let pace_min = raw.pace_min.unwrap_or_default();
                let pace_max = raw.pace_max.unwrap_or_default();
                Some(SportExtra::Running(Running { speed_avg, cadence_avg, stride_length_avg, steps_total, pace_min, pace_max }))
            }
            _ => None,
        }
    }
}

impl Sport {
    pub fn parse_from_xml(xml: &str) -> Result<Sport, String> {
        let data: SportXML = xml_de::from_str(xml).map_err(|e| format!("XML解析失败: {}", e))?;
        let ts = parse_timestamp(&data.start_time)?;
        let extra = data
            .extra
            .and_then(|raw| SportExtra::from_raw(data.r#type, raw));
        let tracks = data
            .tracks
            .into_iter()
            .map(|t: XMLSportTrack| Track {
                distance_meter: t.distance_meter,
                duration_second: t.duration_second,
                pace_average: t.pace_average,
                extra: t.extra.and_then(|raw| SportExtra::from_raw(data.r#type, raw)),
            })
            .collect::<Vec<_>>();
        Ok(Sport {
            id: 0,
            r#type: data.r#type,
            start_time: ts,
            calories: data.calories,
            distance_meter: data.distance_meter,
            duration_second: data.duration_second,
            heart_rate_avg: data.heart_rate_avg,
            heart_rate_max: data.heart_rate_max,
            pace_average: data.pace_average,
            extra,
            tracks,
        })
    }
}

pub fn parse_timestamp(s: &str) -> Result<i64, String> {
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        if let Some(dt) = Local.from_local_datetime(&ndt).earliest() { return Ok(dt.timestamp()); }
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        if let Some(dt) = Local.from_local_datetime(&ndt).earliest() { return Ok(dt.timestamp()); }
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H") {
        if let Some(dt) = Local.from_local_datetime(&ndt).earliest() { return Ok(dt.timestamp()); }
    }
    if let Ok(nd) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let ndt = nd.and_hms_opt(0, 0, 0).unwrap();
        if let Some(dt) = Local.from_local_datetime(&ndt).earliest() { return Ok(dt.timestamp()); }
    }
    Err("时间格式错误".to_string())
}
