//! Best-effort detection of newer versions published on crates.io.

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use semver::Version;
use serde::{Deserialize, Serialize};

const CRATE_URL: &str = "https://crates.io/api/v1/crates/worldhunt";
const CHECK_TIMEOUT: Duration = Duration::from_secs(2);
const RESPONSE_LIMIT: u64 = 256 * 1024;
const SUCCESS_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const FAILURE_TTL: Duration = Duration::from_secs(6 * 60 * 60);
const DISABLE_VARIABLE: &str = "WORLDHUNT_UPDATE_CHECK";

/// A stable crates.io version newer than the running application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UpdateNotice {
    pub latest: String,
}

/// Starts the entire check on a detached worker and returns immediately.
pub(crate) fn start(current: &str) -> Option<Receiver<UpdateNotice>> {
    if disables(std::env::var_os(DISABLE_VARIABLE).as_deref()) {
        return None;
    }
    let current = Version::parse(current).ok()?;
    spawn_check(move || check(&current))
}

fn spawn_check(
    operation: impl FnOnce() -> Option<UpdateNotice> + Send + 'static,
) -> Option<Receiver<UpdateNotice>> {
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("worldhunt-update-check".to_owned())
        .spawn(move || {
            if let Some(notice) = operation() {
                let _ = sender.send(notice);
            }
        })
        .ok()?;
    Some(receiver)
}

fn check(current: &Version) -> Option<UpdateNotice> {
    check_with(
        current,
        cache_path().as_deref(),
        SystemTime::now(),
        fetch_latest,
    )
}

fn check_with(
    current: &Version,
    cache: Option<&Path>,
    now: SystemTime,
    fetch: impl FnOnce() -> Result<Version, ()>,
) -> Option<UpdateNotice> {
    let latest = if let Some(cached) = cache.and_then(|path| fresh_cache(path, now)) {
        cached.latest.and_then(|value| Version::parse(&value).ok())
    } else {
        match fetch() {
            Ok(latest) => {
                if let Some(path) = cache {
                    write_cache(path, Some(&latest), now);
                }
                Some(latest)
            }
            Err(()) => {
                if let Some(path) = cache {
                    write_cache(path, None, now);
                }
                None
            }
        }
    }?;

    (latest > *current).then(|| UpdateNotice {
        latest: latest.to_string(),
    })
}

fn fetch_latest() -> Result<Version, ()> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(CHECK_TIMEOUT))
        .user_agent(format!(
            "worldhunt/{} (+https://github.com/jfrz38/worldhunt)",
            env!("CARGO_PKG_VERSION")
        ))
        .build();
    let mut response = config
        .new_agent()
        .get(CRATE_URL)
        .header("Accept", "application/json")
        .call()
        .map_err(|_| ())?;
    let payload: CratesResponse = response
        .body_mut()
        .with_config()
        .limit(RESPONSE_LIMIT)
        .read_json()
        .map_err(|_| ())?;
    Version::parse(&payload.crate_metadata.max_stable_version).map_err(|_| ())
}

fn cache_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|directory| directory.join("worldhunt").join("update-check.toml"))
}

fn fresh_cache(path: &Path, now: SystemTime) -> Option<CachedCheck> {
    let cached: CachedCheck = toml::from_str(&fs::read_to_string(path).ok()?).ok()?;
    if cached
        .latest
        .as_deref()
        .is_some_and(|latest| Version::parse(latest).is_err())
    {
        return None;
    }
    let checked_at = UNIX_EPOCH.checked_add(Duration::from_secs(cached.checked_at))?;
    let age = now.duration_since(checked_at).ok()?;
    let ttl = if cached.latest.is_some() {
        SUCCESS_TTL
    } else {
        FAILURE_TTL
    };
    (age < ttl).then_some(cached)
}

fn write_cache(path: &Path, latest: Option<&Version>, now: SystemTime) {
    let Ok(checked_at) = now.duration_since(UNIX_EPOCH) else {
        return;
    };
    let latest = latest.map(ToString::to_string);
    let record = CacheRecord {
        checked_at: checked_at.as_secs(),
        latest: latest.as_deref(),
    };
    let Ok(contents) = toml::to_string(&record) else {
        return;
    };
    if let Some(directory) = path.parent()
        && fs::create_dir_all(directory).is_err()
    {
        return;
    }
    let _ = fs::write(path, contents);
}

fn disables(value: Option<&OsStr>) -> bool {
    value.and_then(OsStr::to_str).is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        )
    })
}

#[derive(Deserialize)]
struct CratesResponse {
    #[serde(rename = "crate")]
    crate_metadata: CrateMetadata,
}

#[derive(Deserialize)]
struct CrateMetadata {
    max_stable_version: String,
}

#[derive(Deserialize)]
struct CachedCheck {
    checked_at: u64,
    #[serde(default)]
    latest: Option<String>,
}

#[derive(Serialize)]
struct CacheRecord<'a> {
    checked_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest: Option<&'a str>,
}

#[cfg(test)]
mod tests;
