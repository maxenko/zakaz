use std::{
    fs::{self, create_dir_all, File},
    path::PathBuf,
    sync::Once,
};

use chrono::Local;
use simplelog::*;

use crate::error::{AppError, AppResult};

pub struct Log;

static LOG_INIT: Once = Once::new();

impl Log {
    fn init() -> AppResult<()> {
        // Create logs directory if it doesn't exist
        let log_dir = "logs";
        create_dir_all(log_dir)?;

        // Generate log file name with date and time
        let log_file_name = format!(
            "logs/zakaz-{}.log",
            Local::now().format("%Y-%m-%d_%H-%M-%S")
        );

        // Configure SimpleLogger
        let log_file = File::create(&log_file_name)
            .map_err(|e| AppError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create log file {}: {}", log_file_name, e)
            )))?;

        CombinedLogger::init(vec![
            WriteLogger::new(
                LevelFilter::Info,
                Config::default(),
                log_file,
            ),
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
        ]).map_err(|e| AppError::Custom(format!("Failed to initialize logger: {}", e)))?;

        // Clean up old log files, keeping only the last 15
        if let Err(e) = Self::cleanup_old_logs(log_dir, 15) {
            // Log cleanup failure is not critical, just log it
            eprintln!("Warning: Failed to cleanup old logs: {}", e);
        }

        Ok(())
    }

    /// Clean up old log files, keeping only the last `keep` files
    fn cleanup_old_logs(dir: &str, keep: usize) -> AppResult<()> {
        let mut logs: Vec<PathBuf> = fs::read_dir(dir)?
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "log"))
            .map(|e| e.path())
            .collect();

        // Sort by modification time - oldest first
        logs.sort_by_key(|path| {
            fs::metadata(path)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

        if logs.len() > keep {
            for path in logs.iter().take(logs.len() - keep) {
                // Ignore individual file deletion errors
                let _ = fs::remove_file(path);
            }
        }

        Ok(())
    }

    fn ensure_initialized() {
        LOG_INIT.call_once(|| {
            if let Err(e) = Self::init() {
                eprintln!("Failed to initialize logger: {}. Logging to stderr.", e);
            }
        });
    }

    fn log(level: LevelFilter, msg: &str) {
        match level {
            LevelFilter::Info => log::info!("{}", msg),
            LevelFilter::Warn => log::warn!("{}", msg),
            LevelFilter::Error => log::error!("{}", msg),
            _ => {},
        }
    }

    pub fn info(msg: &str) {
        Self::ensure_initialized();
        Self::log(LevelFilter::Info, msg);
    }

    #[allow(dead_code)]
    pub fn warn(msg: &str) {
        Self::ensure_initialized();
        Self::log(LevelFilter::Warn, msg);
    }

    pub fn err(msg: &str) {
        Self::ensure_initialized();
        Self::log(LevelFilter::Error, msg);
    }
}

#[macro_export]
macro_rules! inf {
    ($msg:expr) => {
        crate::system::log::Log::info($msg)
    };
    ($( $arg:tt )*) => {
        crate::system::log::Log::info(&format!($( $arg )*))
    };
}

#[macro_export]
macro_rules! wrn {
    ($( $arg:tt )*) => {
        crate::system::log::Log::warn(&format!($( $arg )*))
    };
}

#[macro_export]
macro_rules! err {
    ($msg:expr) => {
        crate::system::log::Log::err($msg)
    };
    ($( $arg:tt )*) => {
        crate::system::log::Log::err(&format!($( $arg )*))
    };
}