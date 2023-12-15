use crate::protobuf_server::services::query_response::Information;
use chrono::Local;
use colored::{ColoredString, Colorize};
use log::SetLoggerError;
use std::io::Write;

#[cfg(feature = "logging")]
/// Sets up the logging
pub fn setup_logger() -> Result<(), SetLoggerError> {
    fn colored_level(level: log::Level) -> ColoredString {
        match level {
            log::Level::Error => level.to_string().red(),
            log::Level::Warn => level.to_string().yellow(),
            log::Level::Info => level.to_string().cyan(),
            log::Level::Debug => level.to_string().blue(),
            log::Level::Trace => level.to_string().magenta(),
        }
    }

    env_logger::Builder::from_env(env_logger::Env::default())
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}:{} {}] - {}",
                Local::now().format("%H:%M:%S").to_string().cyan(),
                record.file().unwrap_or_default(),
                record.line().unwrap_or_default(),
                colored_level(record.level()),
                record.args()
            )
        })
        .try_init()
}

// TODO: Implement a logging for informations to both the CLI and gRPC
/// Gets messages saved for other clients (through gRPC)
pub fn get_messages() -> Vec<Information> {
    unimplemented!()
}
