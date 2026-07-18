use crate::println;
use log::{LevelFilter, Log, Metadata, Record};

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let level = record.level();
        let args = record.args();
        //let mut console: console::Console = console();
        let color = color(level);
        println!("{}[{:>5}]{}", color, level, args);
    }
    fn flush(&self) {}
}

fn color(level: log::Level) -> &'static str {
    match level {
        log::Level::Error => "\x1b[31m",
        log::Level::Warn => "\x1b[93m",
        log::Level::Info => "\x1b[32m",
        log::Level::Debug => "\x1b[36m",
        log::Level::Trace => "\x1b[90m",
    }
}

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN") => LevelFilter::Warn,
        Some("INFO") => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    });
}
