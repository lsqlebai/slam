use quick_xml::de as xml_de;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{NaiveDate, NaiveDateTime, Local, TimeZone};

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
    pub extra: Swimming,
    pub tracks: Vec<Track>,
}

// 移除错误的 SportExtra 结构

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
#[serde(default)]
pub struct Track {
    pub distance_meter: i32,
    pub duration_second: i32,
    pub pace_average: String,
    pub extra: Swimming,
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
    pub test: i32,
}

impl Sport {
    pub fn parse_from_xml(xml: &str) -> Result<Sport, String> {
        let data: SportXML = xml_de::from_str(xml).map_err(|e| format!("XML解析失败: {}", e))?;
        let ts = parse_timestamp(&data.start_time)?;
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
            extra: data.extra,
            tracks: data.tracks,
        })
    }
}

pub const SAMPLE_XML: &'static str = r#"
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

// #[derive(Debug, Serialize, Deserialize, ToSchema)]
// pub struct Running {
// }
/// 使用 serde + quick-xml 自动解析给定的XML字符串为 Swim 结构体

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDateTime, Local, TimeZone};
    #[test]
    fn test_parse_sample_swim() {
        let sport = crate::model::sport::Sport::parse_from_xml(SAMPLE_XML)
            .expect("parse_sample_swim 应该成功");

        assert_eq!(sport.r#type, SportType::Swimming);
        let ndt = chrono::NaiveDateTime::parse_from_str("2025-11-05 20:02:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let expected = chrono::Local.from_local_datetime(&ndt).earliest().unwrap().timestamp();
        assert_eq!(sport.start_time, expected);
        assert_eq!(sport.calories, 200);
        assert_eq!(sport.distance_meter, 1000);
        assert_eq!(sport.duration_second, 600);
        assert_eq!(sport.heart_rate_avg, 120);
        assert_eq!(sport.heart_rate_max, 150);

        assert_eq!(sport.extra.stroke_avg, 20);
        assert_eq!(sport.extra.swolf_avg, 80);

        assert_eq!(sport.id, 0);

        assert_eq!(sport.tracks.len(), 2);

        let t1 = &sport.tracks[0];
        assert_eq!(t1.distance_meter, 25);
        assert_eq!(t1.duration_second, 30);
        assert_eq!(t1.pace_average, "4'00''");
        assert_eq!(t1.extra.main_stroke, "freestyle");
        assert_eq!(t1.extra.stroke_avg, 20);
        assert_eq!(t1.extra.swolf_avg, 80);

        let t2 = &sport.tracks[1];
        assert_eq!(t2.distance_meter, 25);
        assert_eq!(t2.duration_second, 40);
        assert_eq!(t2.pace_average, "4'00''");
        assert_eq!(t2.extra.main_stroke, "freestyle");
        assert_eq!(t2.extra.stroke_avg, 20);
        assert_eq!(t2.extra.swolf_avg, 80);
    }

    #[test]
    fn test_serialize_sport_to_xml() {
        use quick_xml::se as xml_se;

        let sport = Sport {
            id: 1,
            r#type: SportType::Swimming,
            start_time: 1694560000,
            calories: 200,
            distance_meter: 1000,
            duration_second: 600,
            heart_rate_avg: 120,
            heart_rate_max: 150,
            pace_average: "3'59''".to_string(),
            extra: Swimming {
                main_stroke: "freestyle".to_string(),
                stroke_avg: 20,
                swolf_avg: 80,
            },
            tracks: vec![
                Track {
                    distance_meter: 25,
                    duration_second: 30,
                    pace_average: "4'00''".to_string(),
                    extra: Swimming {
                        main_stroke: "freestyle".to_string(),
                        stroke_avg: 20,
                        swolf_avg: 80,
                    },
                },
                Track {
                    distance_meter: 25,
                    duration_second: 40,
                    pace_average: "4'00''".to_string(),
                    extra: Swimming {
                        main_stroke: "freestyle".to_string(),
                        stroke_avg: 20,
                        swolf_avg: 80,
                    },
                },
            ],
        };

        let xml = xml_se::to_string(&sport).expect("serialize sport to xml");
        println!("{}", xml);
        assert!(!xml.is_empty());
    }

    #[test]
    fn test_parse_timestamp_local_full() {
        let s = "2025-11-6 10:22:00";
        let ts = parse_timestamp(s).expect("parse should succeed");
        let ndt = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").expect("format ok");
        let expected = Local
            .from_local_datetime(&ndt)
            .earliest()
            .expect("local mapping should exist")
            .timestamp();
        println!("parsed timestamp: {}", ts);
        assert_eq!(ts, expected);
    }
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
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "swimming" => SportType::Swimming,
            "running" => SportType::Running,
            "cycling" => SportType::Cycling,
            _ => SportType::Unknown,
        }
    }
}

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
    pub extra: Swimming,
    pub tracks: Vec<Track>,
}

fn parse_timestamp(s: &str) -> Result<i64, String> {
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
