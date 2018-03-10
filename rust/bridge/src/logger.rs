use std::ffi::CString;
use time;
use log::{self, Record, Level, Metadata, SetLoggerError};

struct PlatformLogger;

impl log::Log for PlatformLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            log(format!(
                "{} {:<5} [{}] {}\n",
                time::strftime("%Y-%m-%d %H:%M:%S", &time::now()).unwrap(),
                record.level().to_string(),
                record.module_path().unwrap_or_default(),
                record.args()));
        }
    }

    fn flush(&self) {}
}

fn log<T: Into<Vec<u8>>>(message: T) {
    let message = CString::new(message).unwrap();
    unsafe { ::log(message.as_ptr()) };
}

static LOGGER: PlatformLogger = PlatformLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_max_level(Level::Trace.to_level_filter());
    log::set_logger(&LOGGER)
}