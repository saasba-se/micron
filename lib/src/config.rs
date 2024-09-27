use std::net::SocketAddr;

use serde::de::DeserializeOwned;

use crate::{user::Plan, Result};

pub static CONFIG_FILE: &'static str = "saasbase.toml";

/// Application configuration. Defines all the aspects of the application
/// that are to be handled on the `saasbase` level.
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
/// ```rust
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
    pub email: Email,

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
            registration: Default::default(),
            users: vec![],
            phrases: vec![],
            payments: Default::default(),
            company: Default::default(),
            email: Default::default(),
        }
    }
}

/// Loads application config from toml file at default location.
// TODO: recursively search up a few directory levels.
pub fn load<T: DeserializeOwned>() -> Result<T> {
    load_from(CONFIG_FILE)
}

/// Loads application config from toml file at the given path.
pub fn load_from<T: DeserializeOwned>(path: impl AsRef<str>) -> Result<T> {
    let config = config::Config::builder()
        .add_source(config::File::with_name(path.as_ref()))
        .add_source(config::File::with_name(&format!("secret.{}", path.as_ref())).required(false))
        .add_source(
            config::Environment::default()
                .separator("__")
                .prefix_separator("__"),
        )
        .build()?;

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
}

impl Default for Tracing {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: crate::tracing::Mode::default(),
            level: crate::tracing::Level::default(),
            loki_address: "".to_string(),
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Payments {
    pub stripe: Stripe,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Stripe {
    pub secret: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Auth {}

/// OAuth2 authentication configuration, including ability to enable support
/// for different providers.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Oauth {
    pub enabled: bool,
    pub discord: bool,
    pub facebook: OauthEntry,
    pub github: OauthEntry,
    pub google: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct OauthEntry {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Registration {
    pub enabled: bool,
    pub email: bool,
    pub oauth: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Email {
    pub address: String,
    pub smtp_user: String,
    pub smtp_password: String,
    pub smtp_server: String,
    pub smtp_port: u16,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Company {
    pub name: String,
    pub tax_id: String,
    pub country: String,
}
