use anyhow::{Context, Result};
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::env;
use std::fs::{create_dir_all, OpenOptions};
use std::path::Path;
use std::sync::OnceLock;
use log::info;

static LOGGER_INITIALIZED: OnceLock<()> = OnceLock::new();

fn level_from_env() -> LevelFilter {
    match env::var("LOG_LEVEL").ok().as_deref() {
        Some("off") => LevelFilter::Off,
        Some("error") => LevelFilter::Error,
        Some("warn" | "warning") => LevelFilter::Warn,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        Some(_) | None => LevelFilter::Info,
    }
}

/// Initialize thread-safe singleton logger using OnceLock for lock-free concurrent calls.
/// Truncates log file on startup for clean sessions. Environment-aware via LOG_LEVEL variable.
pub fn init<P: AsRef<Path>>(log_file: P) -> Result<()> {
    if LOGGER_INITIALIZED.get().is_some() {
        return Ok(());
    }

    let path = log_file.as_ref();
    if let Some(parent) = path.parent() {
        create_dir_all(parent)
            .with_context(|| format!("failed to create log directory {parent:?}"))?;
    }

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .with_context(|| format!("failed to open log file {path:?}"))?;

    let config = ConfigBuilder::new().build();

    let level = level_from_env();

    WriteLogger::init(level, config, file).context("failed to initialize file logger")?;

    LOGGER_INITIALIZED.set(()).ok();
    info!("Logger initialized (level={level:?}), logging to {path:?}");
    Ok(())
}
