use std::backtrace::Backtrace;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;

use axum::body::HttpBody;
use axum::http::Uri;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Json;
use http::status::StatusCode;
use serde_json::json;
use url::Url;
use uuid::Uuid;

use crate::auth::hash_password;
use crate::routes;

pub type Result<T> = std::result::Result<T, Error>;

// TODO: secret-based public-facing error codes

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub backtrace: Backtrace,
    pub request: Option<Uuid>,
    pub user: Option<Uuid>,
}

impl std::error::Error for Error {}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            backtrace: Backtrace::capture(),
            request: None,
            user: None,
        }
    }

    pub fn new_with(kind: ErrorKind, request: Option<Uuid>, user: Option<Uuid>) -> Self {
        Self {
            kind,
            backtrace: Backtrace::capture(),
            request,
            user,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind.to_string());
        if let Some(user) = self.user {
            write!(f, ", user: {}", user);
        }
        if let Some(request) = self.request {
            write!(f, ", request: {}", request);
        }
        if self.backtrace.status() == std::backtrace::BacktraceStatus::Captured {
            write!(f, ", {}", self.backtrace);
        }
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("unexpected error")]
    StdIoError(#[from] std::io::Error),

    #[error("unexpected error")]
    Unexpected,

    #[error("config error: {0}")]
    ConfigError(#[from] config::ConfigError),

    #[error("failed parsing value from string: {0}")]
    ParsingError(String),

    #[error("http error: {0}")]
    HttpError(#[from] http::Error),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("lettre email error: {0}")]
    LettreEmailError(#[from] lettre::error::Error),
    #[error("lettre smtp  error: {0}")]
    LettreSmtpError(#[from] lettre::transport::smtp::Error),
    #[error("failed parsing email address: {0}")]
    EmailParseError(String),
    #[error("failed sending email: {0}")]
    EmailFailedSend(String),
    #[error("failed sending email through smtp: {0}")]
    EmailBadResponse(String),
    #[error("other error: {0}")]
    Other(String),
    #[error("msg: {0}")]
    Message(String),

    #[error("bad input: {0}")]
    BadInput(String),

    #[error("forbidden")]
    Forbidden,

    #[error("authentication failed: {0}")]
    AuthFailed(String),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("password not set")]
    PasswordNotSet,
    #[error("account disabled")]
    AccountDisabled,
    /// Happens on unauthenticated user trying to access dash routes.
    /// Gets turned into a response redirecting to home page.
    #[error("failed getting token cookie")]
    FailedGettingTokenCookie(Uri),

    #[error("registration currently closed: {0}")]
    RegistrationClosed(String),

    #[error("db error: {0}")]
    DbError(String),

    #[cfg(feature = "sled")]
    #[error("sled db error: {0}")]
    SledError(#[from] sled::Error),
    #[cfg(feature = "sled")]
    #[error("sled conflictable transaction conflict error: {0}")]
    SledConflictableTransactionConflictError(
        #[from] sled::transaction::ConflictableTransactionError,
    ),
    #[cfg(feature = "sled")]
    #[error("sled transaction conflict error: {0}")]
    SledTransactionConflictError(#[from] sled::transaction::TransactionError<Box<ErrorKind>>),

    #[cfg(feature = "redb")]
    #[error("redb error: {0}")]
    RedbError(#[from] redb::Error),
    #[cfg(feature = "redb")]
    #[error("redb database error: {0}")]
    RedbDatabaseError(#[from] redb::DatabaseError),

    #[error("passwordhash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    // // TODO this might have to change in bincode 2
    #[error("bincode decode error: {0}")]
    BincodeError(#[from] bincode::Error),
    #[error("json decode error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("toml decode error: {0}")]
    TomlError(#[from] toml::de::Error),
    #[error("pot decode error: {0}")]
    PotError(#[from] pot::Error),

    #[error("uuid error: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("user with this email already exists: {0}")]
    UserWithEmailAlreadyExists(String),
    #[error("user not found: {0}")]
    UserNotFound(String),
    #[error("user does not have a password set")]
    UserDoesNotHavePassword,

    #[error("infallible?")]
    Infallible(#[from] Infallible),
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Self::new(ErrorKind::Other(e))
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::new(ErrorKind::ReqwestError(e))
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(e: argon2::password_hash::Error) -> Self {
        Self::new(ErrorKind::PasswordHashError(e))
    }
}

impl From<uuid::Error> for Error {
    fn from(e: uuid::Error) -> Self {
        Self::new(ErrorKind::UuidError(e))
    }
}

#[cfg(feature = "sled")]
impl From<sled::Error> for Error {
    fn from(e: sled::Error) -> Self {
        Self::new(ErrorKind::SledError(e))
    }
}

#[cfg(feature = "redb")]
impl From<redb::Error> for Error {
    fn from(e: redb::Error) -> Self {
        Self::new(ErrorKind::RedbError(e))
    }
}
#[cfg(feature = "redb")]
impl From<redb::DatabaseError> for Error {
    fn from(e: redb::DatabaseError) -> Self {
        Self::new(ErrorKind::RedbDatabaseError(e))
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::new(ErrorKind::JsonError(e))
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Self::new(ErrorKind::TomlError(e))
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Self::new(ErrorKind::BincodeError(e))
    }
}

impl From<pot::Error> for Error {
    fn from(e: pot::Error) -> Self {
        Self::new(ErrorKind::PotError(e))
    }
}

impl From<lettre::error::Error> for Error {
    fn from(e: lettre::error::Error) -> Self {
        Self::new(ErrorKind::LettreEmailError(e))
    }
}

impl From<lettre::transport::smtp::Error> for Error {
    fn from(e: lettre::transport::smtp::Error) -> Self {
        Self::new(ErrorKind::LettreSmtpError(e))
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Self::new(ErrorKind::UrlParseError(e))
    }
}

impl From<config::ConfigError> for Error {
    fn from(e: config::ConfigError) -> Self {
        Self::new(ErrorKind::ConfigError(e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::new(ErrorKind::StdIoError(e))
    }
}

impl From<Infallible> for Error {
    fn from(e: Infallible) -> Self {
        Self::new(ErrorKind::Infallible(e))
    }
}

impl From<ErrorKind> for Error {
    fn from(k: ErrorKind) -> Self {
        Self::new(k)
    }
}

/// Implements conversion into html response for all possible error variants.
///
/// # Error message stripping in production
///
/// When compiled with optimizations ("release mode"), responses are stripped
/// or even modified to enhance security.
///
/// Backtrace and additional context information (e.g. user information) are
/// never part of the response and always only available through the
/// application logs.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match &self.kind {
            ErrorKind::Forbidden => StatusCode::FORBIDDEN.into_response(),
            ErrorKind::AuthFailed { .. } => {
                tracing::debug!("{}", self.to_string());
                if cfg!(debug_assertions) {
                    (StatusCode::FORBIDDEN, Html(self.to_string())).into_response()
                } else {
                    Redirect::to("/login").into_response()
                }
            }
            ErrorKind::InvalidCredentials => {
                tracing::debug!("{}", self.to_string());
                (StatusCode::FORBIDDEN, Html(self.to_string())).into_response()
            }
            ErrorKind::PasswordNotSet { .. } => {
                tracing::debug!("{}", self.to_string());
                let msg = if cfg!(debug_assertions) {
                    self.kind.to_string()
                } else {
                    // Don't make it possible for anyone to check if user has
                    // their password set. Return a standard error response
                    // instead.
                    "Invalid credentials".to_string()
                };
                // TODO: immediately send email to the user with a link to set
                // a new password
                (StatusCode::FORBIDDEN, Html(msg)).into_response()
            }
            ErrorKind::AccountDisabled => {
                tracing::debug!("{}", self.to_string());
                // TODO: send email to the user with a link to set password
                (StatusCode::FORBIDDEN, Html(self.to_string())).into_response()
            }

            ErrorKind::FailedGettingTokenCookie(target_url) => {
                tracing::debug!("{}", self.to_string());
                // save redirection target to a cookie
                Redirect::to(&format!("{}?redir={}", "/login", target_url.to_string()))
                    .into_response()
            }
            ErrorKind::RegistrationClosed(e) => {
                tracing::trace!("{}", self.to_string());
                Redirect::to(&format!("{}?msg=Registration closed", routes::LOGIN)).into_response()
            }
            ErrorKind::BadInput(e) => {
                tracing::trace!("{}", self.to_string());
                (StatusCode::BAD_REQUEST, Html(self.to_string())).into_response()
            }
            _ => {
                tracing::error!("{}", self.to_string());
                // let id = tracing::Span::current().field("id").unwrap().to_string();
                let span = tracing::Span::current();
                println!("current span: {:?}", span);
                if let Some(id) = span.field("id") {
                    println!("id: {}", id.to_string());
                    (StatusCode::INTERNAL_SERVER_ERROR, id.to_string()).into_response()
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "").into_response()
                }
            }
        }
    }
}
