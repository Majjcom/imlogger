use chrono::{DateTime, Utc};
use log::{Level, Log, Metadata, Record};

pub use log::{debug, error, info, warn};

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

impl Log for ImLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let ts: DateTime<Utc> = Utc::now();
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
                "{} \x1B{}{}\x1B[0m {} {} - {}",
                ts.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                get_color(record.level()),
                record.level(),
                file,
                record.module_path().unwrap_or("<unknown>"),
                record.args()
            )
        }
    }
    fn flush(&self) {}
}

static IMLOGGER: ImLogger = ImLogger {};

pub fn imloginit(level: log::LevelFilter) {
    log::set_logger(&IMLOGGER).unwrap();
    log::set_max_level(level);
}