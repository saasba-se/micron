use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::ops::Sub;

use axum::Extension;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::PrivateCookieJar;
use chrono::{DateTime, Utc};
use rand::prelude::SliceRandom;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::de::DeserializeOwned;
use tracing::info;
use url::Url;
use uuid::Uuid;

use crate::auth::{self, TokenMeta};
use crate::credits::{Credits, CreditsHistory};
use crate::db::{decode, encode, Database};
use crate::error::{ErrorKind, Result};
use crate::order::{Order, OrderMode, OrderStatus};
use crate::payment::{Payment, Status};
use crate::user::{self, User, UserId};
use crate::Config;

pub fn process_order(order: Order, db: &Database, user_id: UserId) -> Result<()> {
    info!("processing order");

    // subtract the credits from user total
    let mut user = db.get::<User>(user_id)?;
    // println!("delta: {:?}", delta);
    user.credits.available += order.total_cost();
    // println!("credits after cost: {:?}", user.credits.available);
    db.set(&user)?;

    // archive the order
    db.set(&order)?;

    Ok(())
}

pub fn load_toml_config<T: DeserializeOwned>(path: &str) -> Result<T> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    let _ = file.read_to_string(&mut buf)?;
    let out: T = toml::from_str(&buf)?;
    Ok(out)
}

/// Checks if provided token has expired, deleting it if it's expired.
pub fn token_expired(db: &Database, token: &TokenMeta) -> bool {
    if token.is_expired() {
        db.remove(token);
        true
    } else {
        false
    }
}

pub fn find_user_by_email(db: &Database, email: &String) -> Result<User> {
    for user in db.get_collection::<User>()? {
        if &user.email == email {
            return Ok(user);
        }
    }
    Err(ErrorKind::UserNotFound(format!("{}", email)).into())
}

pub fn find_user_by_handle(db: &Database, handle: &String) -> Result<User> {
    for user in db.get_collection::<User>()? {
        if &user.handle == handle {
            return Ok(user);
        }
    }
    Err(ErrorKind::UserNotFound(format!("{}", handle)).into())
}

pub fn create_test_user(db: &Database) -> Result<Uuid> {
    let email = "test@mail.com".to_string();

    // does the test user already exist
    if db
        .get_collection::<User>()?
        .iter()
        .find(|u| u.email == email)
        .is_some()
    {
        return Err(ErrorKind::UserWithEmailAlreadyExists(email.clone()).into());
    }

    let mut user = User::new(db)?;
    user.is_admin = true;
    user.is_disabled = false;
    user.email = email;
    user.email_confirmed = true;
    user.password_hash = Some(auth::hash_password("test")?);
    user.name = "Test User".to_string();
    user.handle = "testuser12".to_string();
    user.plan = user::Plan {
        name: "Enterprise".to_string(),
        ..Default::default()
    };
    user.credits = Credits {
        available: Decimal::ONE_HUNDRED,
        history: Default::default(),
    };
    user.notifications = user::UserNotifications {
        updates: vec![user::UpdateNotification {
            title: "Test Notification".to_string(),
            description: "Fake notification created when spawning the test user".to_string(),
            r#type: user::UpdateNotificationType::Info,
            time: Utc::now(),
            url: "/".to_string(),
            read: false,
        }],
        alerts: vec![],
    };
    user.settings = Default::default();
    user.activities = user::UserActivities {
        list: vec![user::UserActivity {
            time: Utc::now(),
            category: user::UserActivityCategory::Payment,
            message: "Paid some fake amount of money".to_string(),
        }],
    };

    db.set(&user)?;
    let user_id = user.id;

    // insert some orders assigned to the user
    let order1 = Order {
        id: Uuid::new_v4(),
        user: user_id,
        time: Utc::now()
            .checked_sub_signed(chrono::Duration::hours(5))
            .unwrap(),
        status: OrderStatus::Completed {
            time: Utc::now()
                .checked_sub_signed(chrono::Duration::hours(4))
                .unwrap(),
        },
        mode: OrderMode::Manual,
        items: vec![],
    };
    db.set(&order1)?;
    let order2 = Order {
        id: Uuid::new_v4(),
        user: user_id,
        time: Utc::now()
            .checked_sub_signed(chrono::Duration::hours(15))
            .unwrap(),
        status: OrderStatus::Failed,
        mode: OrderMode::Manual,
        items: vec![],
    };
    db.set(&order2)?;

    Ok(user_id)
}

pub fn get_random_phrase(config: &Config) -> String {
    config
        .phrases
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone()
}

/// Creates an easily bindable address using the `0.0.0.0` meta-address and
/// any available port.
pub fn get_available_address() -> Result<SocketAddr> {
    let listener = std::net::TcpListener::bind("0.0.0.0:0")?;
    let addr = listener.local_addr()?;
    Ok(addr)
}

/// Re-generates cookie key and stores it in place of the old one.
pub fn regen_cookie_key(db: &Database) -> Result<()> {
    let key = cookie::Key::generate();
    db.set_raw_at("cookie_keys", &key.master(), Uuid::nil())
        .unwrap();

    Ok(())
}
