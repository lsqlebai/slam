use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use super::sport::SportType;
use chrono::{NaiveDate, NaiveDateTime, Local, TimeZone};

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
#[serde(default, rename = "sport")]
pub struct SportXML {
    pub r#type: SportType,
    pub start_time: String,
    pub calories: i32,
    pub distance_meter: i32,
    pub duration_second: i32,
    pub heart_rate_avg: i32,
    pub heart_rate_max: i32,
    pub pace_average: String,
    pub extra: Option<XMLSportExtra>,
    pub tracks: Vec<XMLSportTrack>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
#[serde(default)]
pub struct XMLSportTrack {
    pub distance_meter: i32,
    pub duration_second: i32,
    pub pace_average: String,
    pub extra: Option<XMLSportExtra>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct XMLSportExtra {
    pub main_stroke: Option<String>,
    pub stroke_avg: Option<i32>,
    pub swolf_avg: Option<i32>,
    pub speed_avg: Option<f32>,
    pub cadence_avg: Option<i32>,
    pub stride_length_avg: Option<i32>,
    pub steps_total: Option<i32>,
    pub pace_min: Option<String>,
    pub pace_max: Option<String>,
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

pub const SAMPLE_XML_SWIMMING: &str = r#"
    <sport>
        <type>Swimming</type>
        <start_time>2025-11-05 20:02:00</start_time>
        <calories>200</calories>
        <distance_meter>1000</distance_meter>
        <duration_second>600</duration_second>
        <heart_rate_avg>120</heart_rate_avg>
        <heart_rate_max>150</heart_rate_max>
        <pace_average>3'59''</pace_average>
        <extra>
            <main_stroke>freestyle</main_stroke>
            <stroke_avg>20</stroke_avg>
            <swolf_avg>80</swolf_avg>
        </extra>
        <tracks>
            <distance_meter>25</distance_meter>
            <duration_second>30</duration_second>
            <pace_average>4'00''</pace_average>
            <extra>
                <main_stroke>freestyle</main_stroke>
                <stroke_avg>20</stroke_avg>
                <swolf_avg>80</swolf_avg>
            </extra>
        </tracks>
        <tracks>
            <distance_meter>25</distance_meter>
            <duration_second>40</duration_second>
            <pace_average>4'00''</pace_average>
            <extra>
                <main_stroke>freestyle</main_stroke>
                <stroke_avg>20</stroke_avg>
                <swolf_avg>80</swolf_avg>
            </extra>
        </tracks>
    </sport>
"#;

pub const SAMPLE_XML_RUNNING: &str = r#"
    <sport>
        <type>Running</type>
        <start_time>2025-05-17 20:28:00</start_time>
        <calories>291</calories>
        <distance_meter>4820</distance_meter>
        <duration_second>1872</duration_second>
        <heart_rate_avg>158</heart_rate_avg>
        <heart_rate_max>172</heart_rate_max>
        <pace_average>6'29''</pace_average>
        <extra>
            <speed_avg>9.26</speed_avg>
            <cadence_avg>164</cadence_avg>
            <stride_length_avg>94</stride_length_avg>
            <steps_total>5122</steps_total>
            <pace_min>6'08''</pace_min>
            <pace_max>6'22''</pace_max>
        </extra>
        <tracks>
            <distance_meter>1000</distance_meter>
            <duration_second>377</duration_second>
            <pace_average>6'17''</pace_average>
        </tracks>
        <tracks>
            <distance_meter>1000</distance_meter>
            <duration_second>368</duration_second>
            <pace_average>6'08''</pace_average>
        </tracks>
        <tracks>
            <distance_meter>1000</distance_meter>
            <duration_second>378</duration_second>
            <pace_average>6'18''</pace_average>
        </tracks>
        <tracks>
            <distance_meter>1000</distance_meter>
            <duration_second>382</duration_second>
            <pace_average>6'22''</pace_average>
        </tracks>
        <tracks>
            <distance_meter>820</distance_meter>
            <duration_second>367</duration_second>
            <pace_average>6'07''</pace_average>
        </tracks>
    </sport>
"#;
