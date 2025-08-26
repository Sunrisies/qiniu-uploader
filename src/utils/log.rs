use anyhow::{Context, Result};
use log::LevelFilter;

use crate::LOG_FILE;
pub fn setup_logging() -> Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] {} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Info)
        .chain(fern::log_file(LOG_FILE)?)
        .apply()
        .context("Failed to initialize logging")?;

    Ok(())
}
