use async_trait::async_trait;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, Set, Statement, DbBackend, ConnectionTrait};
use crate::dao::entities::{avatars, users};
use crate::model::user::{User, UserInfo};
use crate::dao::idl::UserDao;
use super::Repository;

#[async_trait]
impl UserDao for Repository {
    async fn insert(&self, user: User) -> Result<i32, String> {
        let mut am: users::ActiveModel = Default::default();
        am.name = Set(user.name);
        am.password = Set(user.password);
        am.nickname = Set(user.nickname);
        am.avatar = Set(String::new());
        let res = users::Entity::insert(am)
            .exec(&self.conn)
            .await
            .map_err(|e| format!("插入用户失败: {}", e))?;
        Ok(res.last_insert_id)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<UserInfo>, String> {
        let user = users::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(|e| format!("查询用户失败: {}", e))?;
        let Some(user) = user else { return Ok(None); };
        let avatar = avatars::Entity::find()
            .filter(avatars::Column::Uid.eq(id))
            .one(&self.conn)
            .await
            .map_err(|e| format!("查询头像失败: {}", e))?;
        let avatar_data = avatar.map(|a| a.data).unwrap_or_else(|| user.avatar);
        Ok(Some(UserInfo { nickname: user.nickname, avatar: avatar_data }))
    }

    async fn login(&self, name: &str, password: &str) -> Result<Option<User>, String> {
        let user = users::Entity::find()
            .filter(users::Column::Name.eq(name.to_string()))
            .filter(users::Column::Password.eq(password.to_string()))
            .one(&self.conn)
            .await
            .map_err(|e| format!("查询用户失败: {}", e))?;
        Ok(user.map(|u| User { id: u.id, name: u.name, password: u.password, nickname: u.nickname }))
    }

    async fn set_avatar(&self, uid: i32, base64: String) -> Result<(), String> {
        // 使用原生 SQL 实现 SQLite 的 UPSERT 逻辑
        let sql = format!(
            "INSERT INTO avatars (uid, data, mime, created_at, updated_at)\n             VALUES ({}, '{}', 'image/jpeg', strftime('%s','now'), strftime('%s','now'))\n             ON CONFLICT(uid) DO UPDATE SET data = excluded.data, mime = excluded.mime, updated_at = strftime('%s','now')",
            uid,
            base64.replace("'", "''"),
        );
        self.conn
            .execute(Statement::from_string(DbBackend::Sqlite, sql))
            .await
            .map_err(|e| format!("更新头像失败: {}", e))?;
        Ok(())
    }
}
