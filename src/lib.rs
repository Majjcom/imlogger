use std::sync::OnceLock;

use chrono::Local;
use log::{Level, Log, Metadata, Record};

pub use log::{debug, error, info, warn, trace, LevelFilter};

pub struct ImLogger;

fn get_color(level: Level) -> &'static str {
    match level {
        Level::Error => "[31m",
        Level::Info => "[32m",
        Level::Debug => "[33m",
        Level::Warn => "[35m",
        Level::Trace => "[0m",
    }
}

/// Records whose target or module path starts with any of `prefixes` are
/// excluded from logging. Error-level records are never excluded so that
/// genuine failures from a filtered source remain visible.
fn record_excluded(prefixes: &[&str], record: &Record) -> bool {
    if record.level() == Level::Error {
        return false;
    }
    let target = record.target();
    let module = record.module_path().unwrap_or("");
    prefixes
        .iter()
        .any(|prefix| target.starts_with(prefix) || module.starts_with(prefix))
}

fn metadata_excluded(prefixes: &[&str], metadata: &Metadata) -> bool {
    if metadata.level() > Level::Error {
        let target = metadata.target();
        if prefixes.iter().any(|prefix| target.starts_with(prefix)) {
            return true;
        }
    }
    false
}

struct LoggerState {
    excluded: &'static [&'static str],
}

static STATE: OnceLock<LoggerState> = OnceLock::new();

impl Log for ImLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let Some(state) = STATE.get() else {
            return false;
        };
        metadata.level() <= log::max_level()
            && !metadata_excluded(state.excluded, metadata)
    }
    fn log(&self, record: &Record) {
        let Some(state) = STATE.get() else {
            return;
        };
        if record.level() > log::max_level() || record_excluded(state.excluded, record) {
            return;
        }
        let ts = Local::now();
        let file: &str;
        #[cfg(debug_assertions)]
        {
            file = record.file().unwrap_or("<unknown>");
        }
        #[cfg(not(debug_assertions))]
        {
            file = "";
        }
        println!(
            "{} \x1B{}{:<5}\x1B[0m {} {} - {}",
            ts.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            get_color(record.level()),
            record.level(),
            file,
            record.module_path().unwrap_or("<unknown>"),
            record.args()
        )
    }
    fn flush(&self) {}
}

static IMLOGGER: ImLogger = ImLogger {};

/// Initialize the logger, dropping non-error records whose target or module
/// path starts with any of `excluded` (e.g. `&["rocket"]` to silence the
/// framework's request/launch logs).
pub fn imloginit_with(level: LevelFilter, excluded: &'static [&'static str]) {
    let _ = STATE.set(LoggerState { excluded });
    log::set_logger(&IMLOGGER).unwrap();
    log::set_max_level(level);
}

pub fn imloginit(level: LevelFilter) {
    imloginit_with(level, &[]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(level: Level, target: &'static str, module: &'static str) -> Record<'static> {
        Record::builder()
            .level(level)
            .target(target)
            .module_path_static(Some(module))
            .args(format_args!("test"))
            .build()
    }

    #[test]
    fn excludes_matching_target_and_module() {
        let prefixes = &["rocket"];
        assert!(record_excluded(
            prefixes,
            &record(Level::Info, "rocket::server", "rocket::server")
        ));
        assert!(record_excluded(
            prefixes,
            &record(Level::Warn, "rocket::launch", "some_crate::main")
        ));
        assert!(record_excluded(
            prefixes,
            &record(Level::Debug, "app", "rocket::server")
        ));
    }

    #[test]
    fn keeps_non_matching_records() {
        let prefixes = &["rocket"];
        assert!(!record_excluded(
            prefixes,
            &record(Level::Info, "cyber_agent_core", "cyber_agent_core::core")
        ));
        assert!(!record_excluded(
            prefixes,
            &record(Level::Warn, "hyper::server", "hyper::server")
        ));
    }

    #[test]
    fn never_excludes_error_level() {
        let prefixes = &["rocket"];
        assert!(!record_excluded(
            prefixes,
            &record(Level::Error, "rocket::server", "rocket::server")
        ));
    }

    #[test]
    fn empty_prefix_list_keeps_everything() {
        let prefixes: &[&str] = &[];
        assert!(!record_excluded(
            prefixes,
            &record(Level::Info, "rocket::server", "rocket::server")
        ));
    }

    #[test]
    fn metadata_filter_matches_target() {
        let metadata = Record::builder()
            .level(Level::Info)
            .target("rocket::launch")
            .args(format_args!("test"))
            .build()
            .metadata()
            .clone();
        assert!(metadata_excluded(&["rocket"], &metadata));
    }
}
