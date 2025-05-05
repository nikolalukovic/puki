use std::sync::Mutex;

use log::{Level, Log, Metadata, Record};
use syslog::{Facility, Formatter3164, Logger, LoggerBackend};

pub struct DualLogger {
    syslog_logger: Mutex<Logger<LoggerBackend, Formatter3164>>,
    stdout_enabled: bool,
}

impl Log for DualLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!("{}:{}", record.level(), record.args());

            let mut logger = self.syslog_logger.lock().unwrap();

            match record.level() {
                Level::Info => logger.info(&message).unwrap(),
                Level::Error => logger.err(&message).unwrap(),
                Level::Warn => logger.warning(&message).unwrap(),
                Level::Debug => logger.debug(&message).unwrap(),
                Level::Trace => logger.debug(&message).unwrap(),
            }

            if self.stdout_enabled {
                println!("{}", message);
            }
        }
    }

    fn flush(&self) {}
}

pub fn init_logger(stdout_enabled: bool) {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "puki".into(),
        pid: std::process::id(),
    };

    let syslog_logger = syslog::unix(formatter).expect("Could not connect to syslog");

    let logger = Box::new(DualLogger {
        syslog_logger: Mutex::new(syslog_logger),
        stdout_enabled,
    });

    log::set_boxed_logger(logger).unwrap();
    log::set_max_level(log::LevelFilter::Info);
}
