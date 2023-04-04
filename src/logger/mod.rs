#[cfg(feature = "log")]
mod inner {
    use std::collections::HashMap;
    use std::string::ToString;
    use log::{Level, Metadata, Record};
    use colored::Colorize;

    lazy_static::lazy_static! {
        pub static ref LOGGER_COLORS: HashMap<Level, String> =  HashMap::from([
            (Level::Debug, "green".to_string()),
            (Level::Info, "blue".to_string()),
            (Level::Warn, "yellow".to_string()),
            (Level::Error, "red".to_string()),
        ]);
    }


    pub static CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger;
    pub struct ConsoleLogger;

    impl log::Log for ConsoleLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= Level::Debug
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) && record.module_path().unwrap_or("").starts_with("bodoConnect::") {
                let level = format!("{:>7}", record.level());
                eprintln!(
                    "{}: {}",
                    match LOGGER_COLORS.get(&record.level()) {
                        Some(c) => level.color(&**c).to_string(),
                        None => level
                    },
                    record.args())
            }
        }

        fn flush(&self) {}
    }
}

#[cfg(feature = "log")]
pub use inner::ConsoleLogger;

#[not(cfg(feature = "log"))]
mod dummy;