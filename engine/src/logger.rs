use anyhow::{Context, Result};
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::fs::{create_dir_all, OpenOptions};
use std::path::Path;
use std::sync::OnceLock;
use log::info;

static LOGGER_INITIALIZED: OnceLock<()> = OnceLock::new();

/// Initialize a simple file-backed logger (singleton).
/// Calling multiple times is safe and will be a no-op after the first call.
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
        .append(true)
        .write(true)
        .open(path)
        .with_context(|| format!("failed to open log file {path:?}"))?;

    let config = ConfigBuilder::new()
        .build();

    WriteLogger::init(LevelFilter::Info, config, file)
        .context("failed to initialize file logger")?;

    LOGGER_INITIALIZED.set(()).ok();
    info!("Logger initialized, logging to {path:?}");
    Ok(())
}
