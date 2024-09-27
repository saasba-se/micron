//! Collection of utilities for interfacing with the saasbase platform.
//!
//! Interacting with the platform from an application allows programmatic
//! access to existing deployments. This means we can query other applications
//! for arbitrary information and mutate them if needed. This allows for quite
//! deep integration between otherwise separate saasbase applications.

/// Platform API is the same as `saasbase::api` as the platform is a saasbase
/// application itself.
pub use super::api;

use crate::Result;

/// Authenticates with the saasbase platform using provided credentials.
/// On success returns a token that can be used to talk to saasbase platform.
pub async fn authenticate(email: String, password: String) -> Result<String> {
    let request = api::AuthRequest {
        email,
        password,
        scope: api::AuthScope::Public,
        term: api::AuthDuration::Long,
        // TODO add checksum to the context
        context: "saasbase".to_string(),
    };
    let response: api::AuthResponse = reqwest::Client::new()
        // .post("https://saasba.se/api/auth")
        .post("http://127.0.0.1:8000/api/auth")
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    Ok(response.token)
}

pub async fn store_token(token: String) -> Result<()> {
    todo!()
}

pub async fn restore_token() -> Result<String> {
    todo!()
}
