use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::model::sport::{Track, SportExtra, Swimming, Running};

// sports table entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sports")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub uid: i32,
    #[sea_orm(column_name = "type")]
    pub type_: String,
    pub start_time: i64,
    pub calories: i32,
    pub distance_meter: i32,
    pub duration_second: i32,
    pub heart_rate_avg: i32,
    pub heart_rate_max: i32,
    pub pace_average: String,
    pub extra: String,
    pub tracks: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub mod users {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "users")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub password: String,
        pub nickname: String,
        pub avatar: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod avatars {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "avatars")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub uid: i32,
        pub data: String,
        pub mime: String,
        pub created_at: i64,
        pub updated_at: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

// Moved from model/sport_db.rs
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
