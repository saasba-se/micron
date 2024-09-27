use std::time::Duration;

use serde::{Deserialize, Serialize};

// TODO make into set of enum variants for more granular control
/// Defines the scope of access for resulting access token.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthScope {
    /// Restricted to publicly visible information
    Public,
    Complete,
}

/// Defines the length-of-life of resulting access token.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum AuthDuration {
    /// 1 hour
    Short,
    /// 1 day
    Medium,
    /// 30 days
    Long,
}

// conversion method for making `AuthDuration` into an actual `Duration`
impl Into<Duration> for AuthDuration {
    fn into(self) -> Duration {
        match self {
            Self::Short => Duration::from_secs(60 * 60),
            Self::Medium => Duration::from_secs(24 * 60 * 60),
            Self::Long => Duration::from_secs(30 * 24 * 60 * 60),
        }
    }
}

/// Auth request to be sent to `api/auth` endpoint.
///
/// If credentials match a new access token will be generated and sent back
/// to the caller. The token will be generated using information provided in
/// the request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
    /// Scope of information that shall be available when using the resulting
    /// token.
    pub scope: AuthScope,
    /// General duration for which the resulting token shall be valid.
    pub term: AuthDuration,
    /// Context in which the resulting token is being requested, e.g.
    /// application name or other additional information.
    pub context: String,
}

/// Auth response to be sent back from `api/auth` endpoint.
///
/// Contains a newly generated access token that can be used with subsequent
/// API requests (as bearer token).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeResponse {
    pub email: String,
    pub name: String,
    pub plan: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreditsAddRequest {
    pub amount: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreditsAddResponse {
    pub resulting_balance: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeployQuery {
    pub name: String,
    pub address: String,
}
