use std::{io::Write as _, sync::Mutex};

use log::{Level, Log, Metadata, Record};
use syslog::{Facility, Formatter3164, Logger, LoggerBackend};

pub struct PukiLogger {
    syslog_logger: Mutex<Logger<LoggerBackend, Formatter3164>>,
    stdout_enabled: bool,
}

impl Log for PukiLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!("{}:{}", record.level(), record.args());

            if self.stdout_enabled {
                println!("{}", message);
            } else {
                let mut logger = self.syslog_logger.lock().unwrap();

                match record.level() {
                    Level::Info => logger.info(record.args()).unwrap(),
                    Level::Error => logger.err(record.args()).unwrap(),
                    Level::Warn => logger.warning(record.args()).unwrap(),
                    Level::Debug => logger.debug(record.args()).unwrap(),
                    Level::Trace => logger.debug(record.args()).unwrap(),
                }
            }
        }
    }

    fn flush(&self) {
        if self.stdout_enabled {
            let _ = std::io::stdout().flush();
        }
    }
}

pub fn init_logger(stdout_enabled: bool) {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "puki".into(),
        pid: std::process::id(),
    };

    let syslog_logger = syslog::unix(formatter).expect("Could not connect to syslog");

    let logger = Box::new(PukiLogger {
        syslog_logger: Mutex::new(syslog_logger),
        stdout_enabled,
    });

    log::set_boxed_logger(logger).unwrap();
    log::set_max_level(log::LevelFilter::Info);
}
