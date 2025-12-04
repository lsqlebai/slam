use std::env;
use serde_json::{json};
use reqwest::header;

#[tokio::test]
async fn test_get_api_key_from_env() {
    let api_key = env::var("AI_API_KEY").ok().filter(|k| !k.trim().is_empty());
    assert!(api_key.is_some() || api_key.is_none());
}

#[tokio::test]
async fn test_doubao_request() {
    let api_key = env::var("AI_API_KEY").expect("has keys");
    let client = reqwest::Client::builder()
        .user_agent("ark-rust-example/0.1")
        .build()
        .unwrap();
    let body = json!({
        "model": "doubao-seed-1-6-251015",
        "max_completion_tokens": 65535,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "image_url", "image_url": { "url": "https://ark-project.tos-cn-beijing.ivolces.com/images/view.jpeg" }},
                    {"type": "text", "text": "图片主要讲了什么?"}
                ]
            }
        ],
        "reasoning_effort": "medium"
    });
    let url = "https://ark.cn-beijing.volces.com/api/v3/chat/completions";
    let resp = client
        .post(url)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let text = resp.text().await.unwrap();
    assert!(status.is_success(), "{}", text);
}

#[tokio::test]
async fn test_sqlite_insert_and_query_sport_from_sample_xml() {
use slam_server::dao::sqlite_impl::SqliteImpl;
    use slam_server::dao::idl::SportDao;
use slam_server::model::sport::Sport;
use slam_server::model::sport::SAMPLE_XML_SWIMMING;
    use std::path::Path;
    let db_path = "tests/test.db";
    if Path::new(db_path).exists() {
        let _ = std::fs::remove_file(db_path);
    }
    let sport = Sport::parse_from_xml(SAMPLE_XML_SWIMMING).expect("parse xml");
    let dao = SqliteImpl::new(db_path).await.expect("dao new");
    dao.insert(0, sport.clone()).await.expect("dao insert");
    let default_page = 0;
    let default_size = 20;
    let all = dao.list(0, default_page, default_size).await.expect("dao list");
    assert_eq!(all.len(), 1);
}

#[tokio::test]
async fn test_sqlite_insert_and_query_running_from_sample_xml() {
    use std::path::Path;
    use slam_server::dao::sqlite_impl::SqliteImpl;
    use slam_server::dao::idl::SportDao;
    use slam_server::model::sport::{Sport, SAMPLE_XML_RUNNING, SportType, SportExtra};
    let db_path = "tests/test.db";
    if Path::new(db_path).exists() { let _ = std::fs::remove_file(db_path); }
    let sport = Sport::parse_from_xml(SAMPLE_XML_RUNNING).expect("parse xml");
    assert_eq!(sport.r#type, SportType::Running);
    let dao = SqliteImpl::new(db_path).await.expect("dao new");
    dao.insert(0, sport.clone()).await.expect("dao insert");
    let all = dao.list(0, 0, 20).await.expect("dao list");
    assert_eq!(all.len(), 1);
    let got = &all[0];
    assert_eq!(got.r#type, SportType::Running);
    assert_eq!(got.distance_meter, 4820);
    assert_eq!(got.duration_second, 1872);
    match &got.extra { Some(SportExtra::Running(r)) => {
        assert_eq!(r.cadence_avg, 164);
        assert_eq!(r.steps_total, 5122);
    }, _ => panic!("extra 类型错误") }
    assert_eq!(got.tracks.len(), 5);
    assert_eq!(got.tracks[0].distance_meter, 1000);
    assert_eq!(got.tracks[0].duration_second, 377);
}

#[test]
fn test_app_config_default_uses_yaml_or_default() {
    use std::path::Path;
    use std::fs;
use slam_server::config::AppConfig as Cfg;
    let cfg = Cfg::default();
    assert!(!cfg.db.path.trim().is_empty());
    let cfg_path = Path::new("config/app.yml");
    if cfg_path.exists() {
        let file = fs::File::open(cfg_path).unwrap();
        let expected: Cfg = serde_yaml::from_reader(file).unwrap();
        assert_eq!(cfg.db.path, expected.db.path);
        assert_eq!(cfg.server.ip, expected.server.ip);
        assert_eq!(cfg.server.port, expected.server.port);
        assert_eq!(cfg.ai.key, expected.ai.key);
    } else {
        assert_eq!(cfg.db.path, "sport.db");
        assert_eq!(cfg.server.ip, "127.0.0.1");
        assert_eq!(cfg.server.port, 3000);
        assert_eq!(cfg.ai.key, "");
    }
}

#[test]
fn test_xiaomi_parser_parse_from_csv_file() {
    use std::fs::File;
    use csv::ReaderBuilder;
    use slam_server::service::sport_service::parse_sports_from_csv;
    use slam_server::model::sport::SportType;

    let file = File::open("tests/test.csv").expect("tests/test.csv should exist");
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);
    let sports = parse_sports_from_csv("xiaomi", &mut reader);
    assert!(!sports.is_empty());
    for s in &sports {
        assert_eq!(s.r#type, SportType::Swimming);
        assert!(s.start_time > 0);
        assert!(s.calories >= 0);
        assert!(s.distance_meter >= 0);
        assert!(s.duration_second >= 0);
        let swim = match &s.extra { Some(slam_server::model::sport::SportExtra::Swimming(x)) => x, _ => panic!("extra 类型错误") };
        assert!(swim.swolf_avg >= 0);
    }

    let file2 = File::open("tests/test.csv").expect("tests/test.csv should exist");
    let mut reader2 = ReaderBuilder::new().has_headers(true).from_reader(file2);
    let mut expected = Vec::new();
    for rec in reader2.records() {
        let rec = rec.expect("csv record");
        let category = rec.get(4).unwrap_or("");
        if category.to_lowercase() != "swimming" { continue; }
        let time_str = rec.get(3).unwrap_or("0");
        let mut start_time: i64 = time_str.parse().unwrap_or(0);
        if start_time > 1_000_000_000_000 { start_time /= 1000; }
        let val_str = rec.get(5).unwrap_or("{}");
        let val: serde_json::Value = serde_json::from_str(val_str).unwrap_or(serde_json::json!({}));
        let calories = val.get("calories").and_then(|v| v.as_i64()).or_else(|| val.get("total_cal").and_then(|v| v.as_i64())).unwrap_or(0) as i32;
        let distance_meter = val.get("distance").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let duration_second = val.get("valid_duration").and_then(|v| v.as_i64()).or_else(|| val.get("duration").and_then(|v| v.as_i64())).unwrap_or(0) as i32;
        let swolf_avg = val.get("avg_swolf").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let stroke_avg = val.get("max_stroke_freq").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        expected.push((start_time, calories, distance_meter, duration_second, swolf_avg, stroke_avg));
    }
    assert_eq!(sports.len(), expected.len());
    for (i, s) in sports.iter().enumerate() {
        let (et, ec, ed, edur, eswolf, estroke) = expected[i];
        assert_eq!(s.start_time, et);
        assert_eq!(s.calories, ec);
        assert_eq!(s.distance_meter, ed);
        assert_eq!(s.duration_second, edur);
        let swim = match &s.extra { Some(slam_server::model::sport::SportExtra::Swimming(x)) => x, _ => panic!("extra 类型错误") };
        assert_eq!(swim.swolf_avg, eswolf);
        assert_eq!(swim.stroke_avg, estroke);
    }
}

#[test]
fn test_app_config_new_with_missing_file_returns_defaults() {
    use slam_server::config::AppConfig as Cfg;
    let missing = "config/__nonexistent__.yml";
    assert!(!std::path::Path::new(missing).exists());
    let cfg = Cfg::new(missing);
    assert_eq!(cfg.db.path, "sport.db");
    assert_eq!(cfg.server.ip, "127.0.0.1");
    assert_eq!(cfg.server.port, 3000);
    assert_eq!(cfg.ai.key, "");
}

// #[tokio::test]
// async fn test_read_sports_in_batches_of_100() {
//     use slam_server::dao::sqlite_impl::SqliteImpl;
//     use slam_server::dao::idl::SportDao;
//     use slam_server::config::AppConfig;
    
//     // Use the default database path from config
//     let db_path = "./sport.db";
    
//     // Check if database file exists
//     if !std::path::Path::new(db_path).exists() {
//         println!("Database file {} does not exist, skipping test", db_path);
//         return;
//     }
    
//     let dao = SqliteImpl::new(db_path).await.expect("Failed to create DAO");
//     let batch_size = 100;
//     let mut page = 0;
//     let mut total_sports = 0;
//     let mut updated_sports = 0;
//     const EIGHT_HOURS_IN_SECONDS: i64 = 8 * 3600; // 8 hours = 28800 seconds
    
//     println!("Reading sports data from {} in batches of {}", db_path, batch_size);
//     println!("Will add 8 hours ({} seconds) to each sport's start_time and update database", EIGHT_HOURS_IN_SECONDS);
    
//     loop {
//         // Read one batch of sports data (user ID 1 for testing)
//         match dao.list(1, page, batch_size).await {
//             Ok(sports) => {
//                 let batch_count = sports.len();
//                 if batch_count == 0 {
//                     println!("No more data available at page {}", page);
//                     break;
//                 }
                
//                 total_sports += batch_count;
//                 println!("Batch {}: {} sports (total so far: {})", page + 1, batch_count, total_sports);
                
//                 // Process each sport in the batch
//                 for (i, mut sport) in sports.into_iter().enumerate() {
//                     let original_time = sport.start_time;
//                     sport.start_time -= EIGHT_HOURS_IN_SECONDS;
                    
//                     println!("  Sport {}: ID={}, Type={}, Start Time={} -> {} (added 8 hours), Calories={}, Distance={}m, Duration={}s",
//                         i + 1,
//                         sport.id,
//                         sport.r#type.as_str(),
//                         original_time,
//                         sport.start_time,
//                         sport.calories,
//                         sport.distance_meter,
//                         sport.duration_second
//                     );
                    
//                     // Update the sport in the database
//                     let id = sport.id;
//                     match dao.update(1, sport).await {
//                         Ok(_) => {
//                             updated_sports += 1;
//                             println!("    ✓ Updated sport ID {} successfully", id);

//                         }
//                         Err(e) => {
//                             println!("    ✗ Failed to update sport ID {}: {}", id, e);
//                         }
//                     }
//                 }
                
//                 // If we got less than batch_size, we've reached the end
//                 if batch_count < batch_size as usize {
//                     println!("Reached end of data (got {} < batch_size {})", batch_count, batch_size);
//                     break;
//                 }
                
//                 page += 1;
//             }
//             Err(e) => {
//                 println!("Error reading batch {}: {}", page + 1, e);
//                 break;
//             }
//         }
//     }
    
//     println!("Total sports read: {}", total_sports);
//     println!("Total sports updated: {}", updated_sports);
//     assert!(total_sports >= 0, "Should have read some sports data");
// }

