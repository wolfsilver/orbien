use super::{AuthFailure, REPLAY_MAX_ENTRIES, REPLAY_TTL_SECS};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct ReplayCache {
    inner: Mutex<HashMap<String, Instant>>,
}

impl ReplayCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accept(&self, digest: &str) -> Result<(), AuthFailure> {
        self.accept_at(digest, Instant::now())
    }

    pub fn accept_at(&self, digest: &str, now: Instant) -> Result<(), AuthFailure> {
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        purge_expired(&mut map, now);

        if map.contains_key(digest) {
            return Err(AuthFailure::Replay);
        }
        if map.len() >= REPLAY_MAX_ENTRIES {
            return Err(AuthFailure::Capacity);
        }

        map.insert(
            digest.to_owned(),
            now + Duration::from_secs(REPLAY_TTL_SECS),
        );
        Ok(())
    }
}

fn purge_expired(map: &mut HashMap<String, Instant>, now: Instant) {
    map.retain(|_, exp| *exp > now);
}
