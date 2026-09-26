use super::{AuthFailure, ReplayCache, AUTH_SKEW_SECS};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn compute_auth_digest(token: &str, timestamp: i64) -> String {
    let mut mac =
        HmacSha256::new_from_slice(token.as_bytes()).expect("HMAC-SHA256 accepts any key length");
    mac.update(timestamp.to_string().as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn verify_auth_digest(
    token: &str,
    auth_digest: &str,
    timestamp: i64,
    now_secs: i64,
    replay: Option<&ReplayCache>,
) -> Result<(), AuthFailure> {
    if token.is_empty() {
        return Ok(());
    }
    if auth_digest.is_empty() {
        return Err(AuthFailure::EmptyDigest);
    }
    if (now_secs - timestamp).abs() > AUTH_SKEW_SECS {
        return Err(AuthFailure::TimestampSkew);
    }

    let expected = hex::decode(auth_digest).map_err(|_| AuthFailure::InvalidDigest)?;
    let mut mac =
        HmacSha256::new_from_slice(token.as_bytes()).map_err(|_| AuthFailure::InvalidDigest)?;
    mac.update(timestamp.to_string().as_bytes());
    mac.verify_slice(&expected)
        .map_err(|_| AuthFailure::InvalidDigest)?;

    if let Some(cache) = replay {
        cache.accept(auth_digest)?;
    }
    Ok(())
}

pub fn verify_login(
    token: &str,
    auth_digest: &str,
    timestamp: i64,
    now_secs: i64,
    replay: &ReplayCache,
) -> Result<(), AuthFailure> {
    verify_auth_digest(token, auth_digest, timestamp, now_secs, Some(replay))
}

pub fn unix_now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
