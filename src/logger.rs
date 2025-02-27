use env_logger;
use log::LevelFilter;
use std::io::Write;

pub fn initialize_logger(log_level: &String) {
    // Set the log level based on the configuration
    let level_filter = match log_level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info, // Default to info if the value is invalid
    };
    // Initialize the logger
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "[{}]: {}", record.level(), record.args()))
        .filter_level(level_filter)
        .init();
}
