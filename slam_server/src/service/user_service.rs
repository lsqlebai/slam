use crate::service::common::ServiceError;
use crate::dao::idl::UserDao;
use crate::model::user::User;
use std::sync::Arc;
use aes::Aes256;
use cbc::Encryptor;
use cipher::{KeyIvInit, BlockEncryptMut, block_padding::Pkcs7};
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose::STANDARD as BASE64_ENGINE, Engine};
use crate::config::SecurityConfig;

pub struct UserService {
    dao: Arc<dyn UserDao + Send + Sync>,
    security: SecurityConfig,
}

impl UserService {
    pub fn new(dao: Arc<dyn UserDao + Send + Sync>, security: SecurityConfig) -> Self {
        Self { dao, security }
    }

    fn derive_key_iv(&self) -> ([u8; 32], [u8; 16]) {
        let mut h1 = Sha256::new();
        h1.update(self.security.key.as_bytes());
        let key_bytes = h1.finalize();
        let mut h2 = Sha256::new();
        h2.update(format!("iv:{}", self.security.salt).as_bytes());
        let iv_full = h2.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        let mut iv = [0u8; 16];
        iv.copy_from_slice(&iv_full[..16]);
        (key, iv)
    }

    fn encrypt_password(&self, pwd: &str) -> String {
        let (key, iv) = self.derive_key_iv();
        let enc = Encryptor::<Aes256>::new(&key.into(), &iv.into());
        let mut buf = pwd.as_bytes().to_vec();
        let block_size = 16;
        let pad_len = block_size - (buf.len() % block_size);
        buf.extend(std::iter::repeat(0u8).take(pad_len));
        let ct = enc.encrypt_padded_mut::<Pkcs7>(&mut buf, pwd.len()).unwrap();
        BASE64_ENGINE.encode(ct)
    }

    pub async fn register(&self, name: String, password: String) -> Result<i32, ServiceError> {
        let encrypted = self.encrypt_password(&password);
        let user = User { id: 0, name, password: encrypted };
        match self.dao.insert(user.clone()).await {
            Ok(uid) => Ok(uid),
            Err(e) => Err(ServiceError { code: 500, message: e }),
        }
    }

    pub async fn login(&self, name: String, password: String) -> Result<i32, ServiceError> {
        let encrypted = self.encrypt_password(&password);
        match self.dao.login(&name, &encrypted).await {
            Ok(Some(u)) => Ok(u.id),
            Ok(None) => Err(ServiceError { code: 401, message: "用户名或密码错误".to_string() }),
            Err(e) => Err(ServiceError { code: 500, message: e }),
        }
    }

    pub async fn get_user(&self, id: i32) -> Result<User, ServiceError> {
        match self.dao.get_by_id(id).await {
            Ok(Some(u)) => Ok(u),
            Ok(None) => Err(ServiceError { code: 404, message: "用户不存在".to_string() }),
            Err(e) => Err(ServiceError { code: 500, message: e }),
        }
    }
}