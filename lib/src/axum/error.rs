use axum::body::HttpBody;
use axum::http::Uri;
use axum::response::{AppendHeaders, Html, IntoResponse, Redirect, Response};
use axum::Json;
use http::header::SET_COOKIE;
use http::StatusCode;

use crate::{routes, Error, ErrorKind};

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
                // Save redirection target to a cookie so that we can perform
                // the final redirection after successful login
                (
                    AppendHeaders([(
                        SET_COOKIE,
                        format!("next={};SameSite=Lax;Secure;Path=/", target_url.to_string()),
                    )]),
                    Redirect::to("/login"),
                )
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
