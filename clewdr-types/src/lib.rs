mod config;
mod reason;
mod usage;

pub use config::ConfigApi;
pub use reason::Reason;
use serde::{Deserialize, Serialize};
pub use usage::UsageBreakdown;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CookieStatusApi {
    pub cookie: String,
    #[serde(default)]
    pub reset_time: Option<i64>,
    #[serde(default)]
    pub count_tokens_allowed: Option<bool>,
    #[serde(default)]
    pub reason: Option<Reason>,
    #[serde(default)]
    pub session_usage: UsageBreakdown,
    #[serde(default)]
    pub weekly_usage: UsageBreakdown,
    #[serde(default)]
    pub weekly_sonnet_usage: UsageBreakdown,
    #[serde(default)]
    pub weekly_opus_usage: UsageBreakdown,
    #[serde(default)]
    pub lifetime_usage: UsageBreakdown,
    pub session_utilization: Option<f64>,
    pub seven_day_utilization: Option<f64>,
    pub seven_day_sonnet_utilization: Option<f64>,
    pub seven_day_opus_utilization: Option<f64>,
    pub session_resets_at: Option<String>,
    pub seven_day_resets_at: Option<String>,
    pub seven_day_sonnet_resets_at: Option<String>,
    pub seven_day_opus_resets_at: Option<String>,
    /// Last detected `first_warning` flag expiry (epoch seconds, UTC).
    #[serde(default)]
    pub first_warning_at: Option<i64>,
    /// Last detected `second_warning` flag expiry (epoch seconds, UTC).
    #[serde(default)]
    pub second_warning_at: Option<i64>,
    /// Last detected `restricted` flag expiry (epoch seconds, UTC).
    #[serde(default)]
    pub restricted_at: Option<i64>,
    /// Last non-rate-limit upstream HTTP error observed using this cookie.
    #[serde(default)]
    pub last_error: Option<CookieLastError>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct CookieLastError {
    pub code: u16,
    pub message: String,
    /// Epoch seconds (UTC) when the error was recorded.
    pub at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UselessCookieApi {
    pub cookie: String,
    pub reason: Option<Reason>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CookieStatusInfoApi {
    #[serde(default)]
    pub valid: Vec<CookieStatusApi>,
    #[serde(default)]
    pub exhausted: Vec<CookieStatusApi>,
    #[serde(default)]
    pub invalid: Vec<UselessCookieApi>,
}
