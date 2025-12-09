use crate::dao::entities::{DbSportExtra, DbSportTrack};
use crate::model::sport::{SportExtra, Track, Swimming};

pub(crate) fn parse_extra_compat(extra_json: &str) -> Option<SportExtra> {
    if extra_json.trim().is_empty() { return None; }
    serde_json::from_str::<Option<DbSportExtra>>(extra_json)
        .map(|o| o.map(SportExtra::from))
        .or_else(|_| serde_json::from_str::<Swimming>(extra_json).map(|s| Some(SportExtra::Swimming(s))))
        .ok()
        .flatten()
}

pub(crate) fn parse_tracks_compat(tracks_json: &str) -> Vec<Track> {
    if tracks_json.trim().is_empty() { return Vec::new(); }
    serde_json::from_str::<Vec<DbSportTrack>>(tracks_json)
        .map(|v| v.into_iter().map(Track::from).collect())
        .or_else(|_| serde_json::from_str::<Vec<Track>>(tracks_json))
        .unwrap_or_default()
}

