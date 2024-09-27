// pub mod avatar;
pub mod subscription;

pub use subscription::Plan;

use std::fmt::{Display, Formatter};
use std::io::{BufWriter, Cursor};
use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use rand::seq::SliceRandom;
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use url::Url;
use uuid::Uuid;

use crate::auth::hash_password;
use crate::credits::Credits;
use crate::db::{decode, encode, Collectable, Database, Identifiable};
use crate::error::{Error, ErrorKind, Result};
use crate::i18n::Language;
use crate::image::{Image, ImageId};
use crate::order::Order;

pub type UserId = Uuid;

/// User data structure.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct User {
    pub id: UserId,

    /// Full name used for things like invoices
    pub full_name: String,
    /// User-chosen name displayed on the platform
    pub display_name: String,

    pub company: String,
    pub website: String,
    pub phone: String,

    pub country: String,
    pub timezone: chrono_tz::Tz,
    // pub timezone: chrono::TimeZone,
    pub currency: Currency,

    pub avatar: ImageId,

    pub registration_date: DateTime<Utc>,

    pub is_admin: bool,
    pub is_disabled: bool,
    pub is_verified: bool,

    pub email: String,
    pub email_confirmed: bool,

    /// Users authenticating with oauth won't have a password set,
    /// unless they choose to set it later, hence the option type.
    pub password_hash: Option<String>,

    pub plan: subscription::Plan,
    pub credits: Credits,

    pub notifications: UserNotifications,
    pub activities: UserActivities,

    pub settings: UserSettings,

    pub completion: usize,
    // pub stripe_customer_id: String,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),

            registration_date: Utc::now(),

            is_admin: false,
            is_disabled: false,
            is_verified: false,

            email: "foo@bar.com".to_string(),
            email_confirmed: false,

            password_hash: None,

            full_name: "Test User".to_string(),
            display_name: "test_user".to_string(),

            company: "".to_string(),
            website: "".to_string(),
            phone: "".to_string(),

            country: "".to_string(),
            timezone: chrono_tz::UTC,
            currency: Currency::USD,

            avatar: Uuid::nil(),

            plan: subscription::Plan::free(),
            credits: Default::default(),
            notifications: Default::default(),
            settings: Default::default(),
            activities: Default::default(),

            completion: 0,
        }
    }
}

impl Collectable for User {
    fn get_collection_name() -> &'static str {
        "user"
    }
}

impl Identifiable for User {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl User {
    pub fn new(db: &Database) -> Result<Self> {
        let colors = [
            (68u8, 153u8, 58u8),
            (40u8, 118u8, 191u8),
            (191u8, 58u8, 40u8),
            (171u8, 169u8, 48u8),
        ];
        let identicon = identicon_rs::new(rand::random::<u16>().to_string())
            .set_color(colors.choose(&mut rand::thread_rng()).unwrap().to_owned())
            .set_border(15)
            .generate_image()
            .unwrap();
        let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
        identicon
            .write_to(&mut buffer, image::ImageFormat::Png)
            .unwrap();
        let mut bytes = buffer.into_inner().unwrap().get_ref().to_vec();
        let image = Image::new(bytes);
        db.set(&image)?;

        let mut user = User::default();
        user.avatar = image.id;

        Ok(user)
    }

    /// Downloads the user image from provided url and adds it to the db.
    /// This is used for example during oauth where we retrieve some image
    /// from an external provider..
    pub async fn set_avatar_from_url(&mut self, db: &Database, url: &str) -> Result<()> {
        let bytes = reqwest::get(url).await?.bytes().await?;
        let image = Image::new(bytes.to_vec());
        db.set(&image)?;
        self.avatar = image.id;

        Ok(())
    }

    pub fn calculate_completion(&mut self) {
        let mut pc = 0;

        if self.is_verified {
            // verified account gets immediate 100 score
            self.completion = 100;
            return;
        }
        if !self.email.is_empty() {
            pc += 20;
        } else {
            // empty email gets immediate 0 score
            self.completion = 0;
            return;
        }
        if self.email_confirmed {
            pc += 20;
        }
        if !self.display_name.is_empty() {
            pc += 10;
        }
        if !self.full_name.is_empty() {
            pc += 10;
        }
        if !self.password_hash.is_none() {
            pc += 10;
        }
        if !self.country.is_empty() {
            pc += 10;
        }

        // TODO: definitely take billing into account here as well

        self.completion = pc;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UpdateNotificationType {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateNotification {
    pub title: String,
    pub description: String,
    pub r#type: UpdateNotificationType,
    pub time: DateTime<Utc>,
    pub url: String,
    pub read: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AlertNotification {
    pub title: String,
    pub description: String,
    pub time: DateTime<Utc>,
    pub read: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserNotifications {
    pub updates: Vec<UpdateNotification>,
    pub alerts: Vec<AlertNotification>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct UserSettings {
    /// Prefer dark theme when visiting dash
    pub dark_mode: bool,

    /// Turn notifications on and off
    // TODO more granular control
    pub dash_notifications: bool,

    /// Turn email notifications on and off
    // TODO more granular control
    pub email_notifications: bool,

    /// Preferred language
    pub language: Language,

    // API related
    /// Enable or disable ability to inspect and manage credits from the level
    /// of the API.
    ///
    /// If user has an automated payment method added, such as a debit card,
    /// they can initiate payments from the API level.
    ///
    /// For safety reasons this functionality is disabled by default.
    pub api_credits: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            dash_notifications: false,
            email_notifications: false,
            language: Language::English,
            api_credits: true,
        }
    }
}

// impl Database {
// /// Adds a new user to the database.
// ///
// /// # Unique email requirement
// ///
// /// Two users sharing the same email address is not allowed.
// ///
// /// # Replace existing user
// ///
// /// This function will replace existing user if the id key already exists
// /// in the database.
// pub fn add_user(&self, id: Uuid, user: &User) -> Result<()> {
//     for _user in self.users.iter() {
//         let (_, user_bytes) = _user?;
//         let _user: User = decode(&user_bytes)?;
//         if _user.email == user.email {
//             return Err(Error::UserWithEmailAlreadyExists(user.email.clone()));
//         }
//     }
//     self.users.insert(id, encode(&user)?);
//     Ok(())
// }
// }

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserActivities {
    pub list: Vec<UserActivity>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserActivity {
    pub time: DateTime<Utc>,
    pub category: UserActivityCategory,
    pub message: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum UserActivityCategory {
    #[default]
    Payment,
}

impl Display for UserActivityCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Payment => write!(f, "payment"),
        }
    }
}

#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Deserialize,
    Serialize,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
)]
pub enum Currency {
    #[default]
    USD,
    EUR,
    PLN,
}
