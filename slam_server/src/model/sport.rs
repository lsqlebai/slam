use quick_xml::de as xml_de;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
pub use crate::model::sport_xml::{SAMPLE_XML_SWIMMING, SAMPLE_XML_RUNNING};
use crate::model::sport_xml::{SportXML, XMLSportExtra, parse_timestamp};

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
#[serde(default, deny_unknown_fields)]
pub struct Swimming {
    pub main_stroke: String,
    pub stroke_avg: i32,
    pub swolf_avg: i32,
}

impl Swimming {
    pub fn new(main_stroke: String, stroke_avg: i32, swolf_avg: i32) -> Self {
        let main_stroke_lower = main_stroke.trim().to_lowercase();
        let main_stroke_normalized = if main_stroke_lower.contains("mix") || main_stroke_lower.contains("混合") {
            "medley"
        } else if main_stroke_lower.contains("free") || main_stroke_lower.contains("自由") {
            "freestyle"
        } else if main_stroke_lower.contains("fly") || main_stroke_lower.contains("蝶") {
            "butterfly"
        } else if main_stroke_lower.contains("breast") || main_stroke_lower.contains("蛙") {
            "breaststroke"
        } else if main_stroke_lower.contains("back") || main_stroke_lower.contains("仰") {
            "backstroke"
        } else if main_stroke_lower.contains("unknown") || main_stroke_lower.contains("未知") {
            "unknown"
        } else {
            "unknown" // Default to unknown for any invalid values
        }.to_string();
        
        Self {
            main_stroke: main_stroke_normalized,
            stroke_avg,
            swolf_avg,
        }
    }
}



#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
#[serde(deny_unknown_fields)]
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
            .map(|t| Track {
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
        let sport = crate::model::sport::Sport::parse_from_xml(SAMPLE_XML_SWIMMING)
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

        let swim = match sport.extra { Some(SportExtra::Swimming(s)) => s, _ => panic!("extra 类型错误") };
        assert_eq!(swim.stroke_avg, 20);
        assert_eq!(swim.swolf_avg, 80);

        assert_eq!(sport.id, 0);

        assert_eq!(sport.tracks.len(), 2);

        let t1 = &sport.tracks[0];
        assert_eq!(t1.distance_meter, 25);
        assert_eq!(t1.duration_second, 30);
        assert_eq!(t1.pace_average, "4'00''");
        let t1e = match &t1.extra { Some(SportExtra::Swimming(s)) => s, _ => panic!("track extra 类型错误") };
        assert_eq!(t1e.main_stroke, "freestyle");
        assert_eq!(t1e.stroke_avg, 20);
        assert_eq!(t1e.swolf_avg, 80);

        let t2 = &sport.tracks[1];
        assert_eq!(t2.distance_meter, 25);
        assert_eq!(t2.duration_second, 40);
        assert_eq!(t2.pace_average, "4'00''");
        let t2e = match &t2.extra { Some(SportExtra::Swimming(s)) => s, _ => panic!("track extra 类型错误") };
        assert_eq!(t2e.main_stroke, "freestyle");
        assert_eq!(t2e.stroke_avg, 20);
        assert_eq!(t2e.swolf_avg, 80);
    }

    #[test]
    fn test_parse_sample_running() {
        let sport = crate::model::sport::Sport::parse_from_xml(SAMPLE_XML_RUNNING)
            .expect("parse_sample_running 应该成功");
        assert_eq!(sport.r#type, SportType::Running);
        assert_eq!(sport.distance_meter, 4820);
        assert_eq!(sport.duration_second, 1872);
        let run = match sport.extra { Some(SportExtra::Running(r)) => r, _ => panic!("extra 类型错误") };
        assert_eq!(run.cadence_avg, 164);
        assert_eq!(run.steps_total, 5122);
        assert_eq!(sport.tracks.len(), 5);
        assert_eq!(sport.tracks[0].distance_meter, 1000);
        assert_eq!(sport.tracks[0].duration_second, 377);
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
            extra: Some(SportExtra::Swimming(Swimming {
                main_stroke: "freestyle".to_string(),
                stroke_avg: 20,
                swolf_avg: 80,
            })),
            tracks: vec![
                Track {
                    distance_meter: 25,
                    duration_second: 30,
                    pace_average: "4'00''".to_string(),
                    extra: Some(SportExtra::Swimming(Swimming {
                        main_stroke: "freestyle".to_string(),
                        stroke_avg: 20,
                        swolf_avg: 80,
                    })),
                },
                Track {
                    distance_meter: 25,
                    duration_second: 40,
                    pace_average: "4'00''".to_string(),
                    extra: Some(SportExtra::Swimming(Swimming {
                        main_stroke: "freestyle".to_string(),
                        stroke_avg: 20,
                        swolf_avg: 80,
                    })),
                },
            ],
        };

        let xml = xml_se::to_string(&sport).expect("serialize sport to xml");
        println!("{}", xml);
        assert!(!xml.is_empty());
    }

    #[test]
    fn test_serialize_running_to_xml() {
        use quick_xml::se as xml_se;
        let sport = Sport {
            id: 2,
            r#type: SportType::Running,
            start_time: 1694560000,
            calories: 291,
            distance_meter: 4820,
            duration_second: 1872,
            heart_rate_avg: 158,
            heart_rate_max: 172,
            pace_average: "6'29''".to_string(),
            extra: Some(SportExtra::Running(Running {
                speed_avg: 9.26,
                cadence_avg: 164,
                stride_length_avg: 94,
                steps_total: 5122,
                pace_min: "6'08''".to_string(),
                pace_max: "6'22''".to_string(),
            })),
            tracks: vec![
                Track { distance_meter: 1000, duration_second: 377, pace_average: "6'17''".to_string(), extra: None },
            ],
        };
        let xml = xml_se::to_string(&sport).expect("serialize running to xml");
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
                Some(SportExtra::Swimming(Swimming::new(main_stroke, stroke_avg, swolf_avg)))
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
