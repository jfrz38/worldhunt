use std::{
    ffi::OsStr,
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use super::*;

static NEXT_TEMPORARY_FILE: AtomicU64 = AtomicU64::new(0);

fn version(value: &str) -> Version {
    Version::parse(value).expect("test version is valid")
}

fn unix(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

fn temporary_cache() -> PathBuf {
    let sequence = NEXT_TEMPORARY_FILE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "worldhunt-update-check-{}-{sequence}.toml",
        std::process::id()
    ))
}

#[test]
fn reports_only_strictly_newer_stable_versions() {
    let current = version("1.2.3");

    let newer = check_with(&current, None, unix(1_000), || Ok(version("1.3.0")));
    let same = check_with(&current, None, unix(1_000), || Ok(version("1.2.3")));
    let older = check_with(&current, None, unix(1_000), || Ok(version("1.2.2")));

    assert_eq!(
        newer,
        Some(UpdateNotice {
            latest: "1.3.0".to_owned()
        })
    );
    assert_eq!(same, None);
    assert_eq!(older, None);
}

#[test]
fn fresh_success_cache_skips_the_fetch() {
    let path = temporary_cache();
    write_cache(&path, Some(&version("1.3.0")), unix(1_000));

    let notice = check_with(&version("1.2.3"), Some(&path), unix(1_001), || {
        panic!("a fresh cache must avoid the network")
    });

    assert_eq!(notice.expect("newer version is reported").latest, "1.3.0");
    let _ = fs::remove_file(path);
}

#[test]
fn expired_cache_is_replaced_with_the_fetched_version() {
    let path = temporary_cache();
    write_cache(&path, Some(&version("1.3.0")), unix(1_000));

    let notice = check_with(
        &version("1.2.3"),
        Some(&path),
        unix(1_000) + SUCCESS_TTL,
        || Ok(version("1.4.0")),
    );

    assert_eq!(notice.expect("newer version is reported").latest, "1.4.0");
    assert_eq!(
        fresh_cache(&path, unix(1_000) + SUCCESS_TTL)
            .and_then(|cache| cache.latest)
            .as_deref(),
        Some("1.4.0")
    );
    let _ = fs::remove_file(path);
}

#[test]
fn failures_are_cached_for_a_shorter_window() {
    let path = temporary_cache();

    assert_eq!(
        check_with(&version("1.2.3"), Some(&path), unix(1_000), || Err(())),
        None
    );
    assert!(fresh_cache(&path, unix(1_000) + FAILURE_TTL - Duration::from_secs(1)).is_some());
    assert!(fresh_cache(&path, unix(1_000) + FAILURE_TTL).is_none());
    assert!(FAILURE_TTL < SUCCESS_TTL);
    let _ = fs::remove_file(path);
}

#[test]
fn invalid_cached_versions_are_refetched() {
    let path = temporary_cache();
    fs::write(&path, "checked_at = 1000\nlatest = 'invalid'").expect("invalid cache is written");

    let notice = check_with(&version("1.2.3"), Some(&path), unix(1_001), || {
        Ok(version("1.3.0"))
    });

    assert_eq!(
        notice.expect("refetched version is reported").latest,
        "1.3.0"
    );
    let _ = fs::remove_file(path);
}

#[test]
fn background_check_returns_before_delivering_a_notice() {
    let (release, wait) = mpsc::channel();
    let receiver = spawn_check(move || {
        wait.recv().expect("test releases the worker");
        Some(UpdateNotice {
            latest: "2.0.0".to_owned(),
        })
    })
    .expect("worker starts");

    assert_eq!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty));
    release.send(()).expect("worker is released");
    assert_eq!(
        receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("worker delivers the notice")
            .latest,
        "2.0.0"
    );
}

#[test]
fn recognized_environment_values_disable_checks() {
    for value in ["0", "false", " OFF ", "No"] {
        assert!(disables(Some(OsStr::new(value))));
    }
    assert!(!disables(Some(OsStr::new("1"))));
    assert!(!disables(None));
}

#[test]
fn deserializes_the_crates_io_version_contract() {
    let response: CratesResponse =
        serde_json::from_str(r#"{"crate":{"max_stable_version":"1.4.0"},"versions":[]}"#)
            .expect("representative crates.io response is accepted");

    assert_eq!(response.crate_metadata.max_stable_version, "1.4.0");
}

#[test]
#[ignore = "live crates.io integration check"]
fn fetches_the_latest_stable_version_from_crates_io() {
    let latest = fetch_latest().expect("crates.io returns a valid stable version");

    assert!(latest >= version(env!("CARGO_PKG_VERSION")));
}
