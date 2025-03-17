use std::{collections::HashSet, net::SocketAddr};

use serde::de::DeserializeOwned;

use crate::{user::Plan, Result};

pub static CONFIG_FILE: &'static str = "micron.toml";

/// Application configuration. Defines all the aspects of the application
/// that are to be handled on the `micron` level.
///
/// # Sensible defaults
///
/// Configuration provided through `Config::default()` allows for quick setup
/// of an application using the recommended workflow. It enables all available
/// features and sets default values for paths, addresses, etc.
///
/// Using the *struct update syntax* one can initialize a new `Config`, making
/// a few changes right in the definition.
///
/// ```ignore
/// let cfg = Config {
///     tracing: Tracing {
///         enabled: false,
///         ..Default::default()
///     },
///     ..Default::default()
/// }
/// ```
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub name: String,
    pub version: String,

    /// Domain name pointing to the machine running the application.
    pub domain: String,
    /// Address on which to serve the application. Defaults to
    /// `127.0.0.1:8080`.
    pub address: SocketAddr,

    pub assets: Assets,
    pub tracing: Tracing,
    pub routers: Routers,

    pub auth: Auth,
    pub oauth: Oauth,

    pub registration: Registration,
    pub comments: Comments,

    pub email: Email,
    pub mailing: Mailing,

    pub payments: Payments,

    /// Information about the company behind the application.
    pub company: Company,

    /// List of available subscription plans.
    pub plans: Vec<Plan>,
    /// List of initial users.
    pub users: Vec<User>,

    /// List of phrases/quotes to be showed randomly on the app pages, because
    /// why not.
    pub phrases: Vec<String>,

    /// Development mode configuration.
    pub dev: DevMode,

    pub init: Init,
    /// Selectively enable/disable pre-made routes
    pub routes: Routes,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            domain: "localhost".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            assets: Assets::default(),
            tracing: Tracing::default(),
            routers: Routers::default(),
            dev: DevMode::default(),
            plans: vec![],
            auth: Auth::default(),
            oauth: Oauth::default(),
            registration: Registration::default(),
            users: vec![],
            phrases: vec![],
            payments: Payments::default(),
            company: Company::default(),
            email: Email::default(),
            mailing: Mailing::default(),
            routes: Routes::default(),
            comments: Comments::default(),
            init: Init::default(),
        }
    }
}

/// Loads application config from toml file at default location.
// TODO: recursively search up a few directory levels.
pub fn load<T: DeserializeOwned>() -> Result<T> {
    load_from(CONFIG_FILE)
}

/// Loads application config from toml file at standard path using provided=
/// name.
///
/// For example for `name` == `micron.toml` we will load both `micron.toml`
/// and `secret.micron.toml` from the main project directory.
pub fn load_from<T: DeserializeOwned>(name: impl AsRef<str>) -> Result<T> {
    let config = config::Config::builder()
        .add_source(config::File::with_name(name.as_ref()))
        .add_source(config::File::with_name(&format!("secret.{}", name.as_ref())).required(false))
        .add_source(
            config::Environment::default()
                .separator("__")
                .prefix_separator("__"),
        )
        .build()?;

    let config: T = config.try_deserialize()?;

    Ok(config)
}

/// Loads application config from multiple toml files at given paths.
pub fn load_from_many<T: DeserializeOwned>(paths: &[impl AsRef<str>]) -> Result<T> {
    let mut builder = config::Config::builder().add_source(
        config::Environment::default()
            .separator("__")
            .prefix_separator("__"),
    );

    for path in paths {
        builder = builder.add_source(config::File::with_name(path.as_ref()));
    }
    let config = builder.build()?;

    let config: T = config.try_deserialize()?;

    Ok(config)
}

/// Intermediate abstraction for initiating a user. Allows things like setting
/// profile image from file during initialization.
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct User {
    #[serde(flatten)]
    pub user: crate::User,
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Assets {
    /// Flag for enabling the asset serving service, serving assets from
    /// filesystem directory based on provided path.
    ///
    /// # Filesystem vs embedded
    ///
    /// When embedding assets into the binary, serving from filesystem should
    /// be turned off.
    pub serve: bool,
    /// Path to the assets directory to be accessed at runtime. Defaults to
    /// `./assets`. Note that the path here is relative to current working
    // directory.
    pub path: String,
}

impl Default for Assets {
    fn default() -> Self {
        Self {
            serve: true,
            path: "assets".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Tracing {
    pub enabled: bool,

    pub mode: crate::tracing::Mode,
    pub level: crate::tracing::Level,

    pub loki_address: String,
    pub loki_token: String,
}

impl Default for Tracing {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: crate::tracing::Mode::default(),
            level: crate::tracing::Level::default(),
            loki_address: "".to_string(),
            loki_token: "".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Routers {
    pub user: bool,
    pub auth: bool,
}

/// NOTE: make sure to disable on production.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DevMode {
    /// Global switch for all dev mode items.
    pub enabled: bool,
    /// Automatic login flag. Includes the email of the user to be logged in.
    pub autologin: Option<String>,
    /// Mocking flag for all the mocking behavior performed by this library.
    pub mock: bool,
    /// Regenerative mocking behavior controls whether to regenerate mocks
    /// that are already present in the database.
    pub mock_regen: bool,
    // mock_filter: Vec<String>,
}

// TODO: allow more granular control.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Init {
    pub enabled: bool,
}

impl Default for Init {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Routes {
    pub enable: Vec<String>,
    pub disable: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Payments {
    pub stripe: Stripe,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Stripe {
    /// Production secret, used with release builds
    pub secret: String,
    /// Test secret, used with debug builds
    pub test_secret: String,

    /// Production signing secret for verifing incoming webhook events
    pub signing_secret: String,
    /// Test signing secret for verifing incoming webhook events
    pub test_signing_secret: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Auth {
    /// Switch defining whether user must have confirmed email to be able
    /// to successfully pass auth.
    ///
    /// If true, users without confirmed email will be presented with ability
    /// to confirm their email. Otherwise the auth will be succesfull, and
    /// any functionality restrictions are up to the application.
    ///
    /// This requirement doesn't have any effect on the oauth process, as
    /// successful oauth requires verified email information from the
    /// third-party provider.
    pub require_confirmed_email: bool,
}

/// OAuth2 authentication configuration, including ability to enable support
/// for different providers.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Oauth {
    pub enabled: bool,
    pub discord: bool,
    pub facebook: OauthEntry,
    pub github: OauthEntry,
    pub google: OauthEntry,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct OauthEntry {
    pub enabled: bool,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Registration {
    /// Top level switch, toggling all registration.
    pub enabled: bool,

    /// Email registration switch.
    pub email: bool,
    pub email_verification: bool,

    /// Oauth2 registration switch, further configured through `config::oauth`.
    pub oauth: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Comments {
    /// Global rate limit per user. Expressed as a number of seconds between
    /// individual comments.
    pub rate_limit: Option<usize>,
}

impl Default for Comments {
    fn default() -> Self {
        Self {
            rate_limit: Some(30),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Email {
    /// Address that the application will use to send emails to users and
    /// non-registered subscribers.
    pub address: String,

    // Smtp server and credentials.
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,

    // Standard message overrides. The tuples are made out of subject,
    // plain body and html body, in that order.
    pub confirmation: Option<(String, String, String)>,
    pub mailing_confirmation: Option<(String, String, String)>,
    pub password_reset: Option<(String, String, String)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Mailing {
    /// All maintained mailing lists that can be subscribed by users and
    /// non-users alike.
    pub lists: HashSet<String>,

    /// Require email confirmation upon subscribing as non-registered user.
    /// For unconfirmed subscriptions the email information will be removed
    /// from the database after a standard period.
    pub confirmation: bool,
}

impl Default for Mailing {
    fn default() -> Self {
        let mut lists = HashSet::new();
        lists.insert("main".to_string());
        Self {
            lists,
            confirmation: true,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct MailingList {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Company {
    pub name: String,
    pub tax_id: String,
    pub country: String,
}
