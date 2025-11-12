use quick_xml::de as xml_de;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Sport {
    #[serde(default)]
    pub id: i32,
    pub r#type: String,
    pub start_time: i64,
    pub calories: i32,
    pub distance_meter: i32,
    pub duration_second: i32,
    pub heart_rate_avg: i32,
    pub heart_rate_max: i32,
    pub pace_average: String,
    pub extra: Swimming,
}

impl Sport {
    pub fn parse_from_xml(xml: &str) -> Result<Sport, String> {
        let data: Sport = xml_de::from_str(xml).map_err(|e| format!("XML解析失败: {}", e))?;
        Ok(data)
    }
}

// #[derive(Debug, Serialize, Deserialize, ToSchema)]
// pub enum SportSpecific {
//     Swimming(Swimming),
//     Running(Running),
// }

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Swimming {
    pub stroke_avg: i32,
    pub swolf_avg: i32,
}

pub const SAMPLE_XML: &'static str = r#"
    <sport>
        <type>Swimming</type>
        <start_time>1694560000</start_time>
        <calories>200</calories>
        <distance_meter>1000</distance_meter>
        <duration_second>600</duration_second>
        <heart_rate_avg>120</heart_rate_avg>
        <heart_rate_max>150</heart_rate_max>
        <pace_average>3'59''</pace_average>
        <extra>
            <stroke_avg>20</stroke_avg>
            <swolf_avg>80</swolf_avg>
        </extra>
    </sport>
"#;

// #[derive(Debug, Serialize, Deserialize, ToSchema)]
// pub struct Running {
// }
/// 使用 serde + quick-xml 自动解析给定的XML字符串为 Swim 结构体

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_sample_swim() {
        let sport = Sport::parse_from_xml(SAMPLE_XML).expect("parse_sample_swim 应该成功");

        assert_eq!(sport.r#type, "Swimming");
        assert_eq!(sport.start_time, 1694560000);
        assert_eq!(sport.calories, 200);
        assert_eq!(sport.distance_meter, 1000);
        assert_eq!(sport.duration_second, 600);
        assert_eq!(sport.heart_rate_avg, 120);
        assert_eq!(sport.heart_rate_max, 150);

        assert_eq!(sport.extra.stroke_avg, 20);
        assert_eq!(sport.extra.swolf_avg, 80);

        // 缺省 id 应为 0
        assert_eq!(sport.id, 0);
    }
}
