mod replay;
mod token;

pub use replay::ReplayCache;
pub use token::{compute_auth_digest, unix_now_secs, verify_auth_digest, verify_login};

use std::fmt;

pub const AUTH_SKEW_SECS: i64 = 180;

pub const REPLAY_TTL_SECS: u64 = (AUTH_SKEW_SECS as u64) * 2;

pub const REPLAY_MAX_ENTRIES: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthFailure {
    EmptyDigest,
    InvalidDigest,
    TimestampSkew,
    Replay,
    Capacity,
}

impl AuthFailure {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EmptyDigest => "empty authentication digest",
            Self::InvalidDigest => "invalid authentication digest",
            Self::TimestampSkew => "timestamp outside allowed window",
            Self::Replay => "authentication digest reused",
            Self::Capacity => "authentication replay cache full",
        }
    }
}

impl fmt::Display for AuthFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
