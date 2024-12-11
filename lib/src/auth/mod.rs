use std::time::{Duration, Instant};

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::routing::{get, post};
use axum::{Extension, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::{AuthDuration, AuthScope};
use crate::db::{decode, encode, Collectable, Database, Identifiable};
use crate::error::{Error, ErrorKind, Result};
use crate::{Config, UserId};

pub mod login;

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    // TODO argon params?
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

pub fn validate_password(password: &[u8], expected_password_hash: &str) -> Result<()> {
    let expected_password_hash = PasswordHash::new(expected_password_hash)
        .map_err(|_| ErrorKind::Other("Failed to parse hash in PHC string format.".to_string()))?;
    Argon2::default().verify_password(password, &expected_password_hash)?;
    // .context("Invalid password.")
    // .map(Error::AuthFailed)?;

    Ok(())
}

pub type TokenId = Uuid;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TokenMeta {
    pub id: Uuid,
    pub user_id: Uuid,
    pub issued_at: DateTime<Utc>,
    pub scope: AuthScope,
    pub duration: AuthDuration,

    pub browser: String,
    pub ip_addr: String,
    pub context: String,
}

impl Collectable for TokenMeta {
    fn get_collection_name() -> &'static str {
        "access_token"
    }
}

impl Identifiable for TokenMeta {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl TokenMeta {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            id: TokenId::new_v4(),
            user_id,
            issued_at: Utc::now(),
            scope: AuthScope::Public,
            duration: AuthDuration::Short,
            context: "".to_string(),
            browser: "Unknown".to_string(),
            ip_addr: "Unknown".to_string(),
        }
    }

    /// Returns true if the token is expired.
    pub fn is_expired(&self) -> bool {
        let delta_time = Utc::now() - self.issued_at;
        let duration: Duration = self.duration.into();
        if delta_time.num_seconds() as u64 > duration.as_secs() {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfirmationKey {
    pub user: UserId,
    pub key: Uuid,
}

impl Collectable for ConfirmationKey {
    fn get_collection_name() -> &'static str {
        "confirmation_keys"
    }
}

impl Identifiable for ConfirmationKey {
    fn get_id(&self) -> Uuid {
        // here the key itself is also the identificator
        self.key
    }
}
