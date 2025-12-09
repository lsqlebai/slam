use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::sport::{Track, SportExtra, Swimming, Running};

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(tag = "type", content = "data")]
pub enum DbSportExtra {
    Swimming(Swimming),
    Running(Running),
}

impl From<SportExtra> for DbSportExtra {
    fn from(e: SportExtra) -> Self {
        match e {
            SportExtra::Swimming(s) => DbSportExtra::Swimming(s),
            SportExtra::Running(r) => DbSportExtra::Running(r),
        }
    }
}

impl From<DbSportExtra> for SportExtra {
    fn from(e: DbSportExtra) -> Self {
        match e {
            DbSportExtra::Swimming(s) => SportExtra::Swimming(s),
            DbSportExtra::Running(r) => SportExtra::Running(r),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
#[serde(default)]
pub struct DbSportTrack {
    pub distance_meter: i32,
    pub duration_second: i32,
    pub pace_average: String,
    pub extra: Option<DbSportExtra>,
}

impl From<Track> for DbSportTrack {
    fn from(t: Track) -> Self {
        DbSportTrack {
            distance_meter: t.distance_meter,
            duration_second: t.duration_second,
            pace_average: t.pace_average,
            extra: t.extra.map(DbSportExtra::from),
}
    }
}

impl From<DbSportTrack> for Track {
    fn from(t: DbSportTrack) -> Self {
        Track {
            distance_meter: t.distance_meter,
            duration_second: t.duration_second,
            pace_average: t.pace_average,
            extra: t.extra.map(SportExtra::from),
}
    }
}
