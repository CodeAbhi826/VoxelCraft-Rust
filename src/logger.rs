// Structured logger using tracing. Mirrors to a ring buffer for the in-game UI.

use parking_lot::Mutex;
use std::sync::OnceLock;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: f64,
    pub level: LogLevel,
    pub scope: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn color(self) -> [f32; 3] {
        match self {
            LogLevel::Debug => [0.42, 0.45, 0.50],
            LogLevel::Info  => [0.90, 0.90, 0.90],
            LogLevel::Warn  => [0.98, 0.75, 0.20],
            LogLevel::Error => [0.97, 0.30, 0.30],
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info  => "INFO",
            LogLevel::Warn  => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

pub struct Logger {
    buffer: Mutex<VecDeque<LogEntry>>,
    capacity: usize,
    subscribers: Mutex<Vec<Box<dyn Fn() + Send + Sync>>>,
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn logger() -> &'static Logger {
    LOGGER.get_or_init(|| Logger {
        buffer: Mutex::new(VecDeque::with_capacity(500)),
        capacity: 500,
        subscribers: Mutex::new(Vec::new()),
    })
}

impl Logger {
    pub fn log(&self, level: LogLevel, scope: &'static str, message: impl Into<String>) {
        let entry = LogEntry {
            timestamp: instant_now(),
            level,
            scope,
            message: message.into(),
        };

        // Also emit to tracing (stderr)
        match level {
            LogLevel::Debug => tracing::debug!(%scope, %entry.message),
            LogLevel::Info  => tracing::info!(%scope, %entry.message),
            LogLevel::Warn  => tracing::warn!(%scope, %entry.message),
            LogLevel::Error => tracing::error!(%scope, %entry.message),
        }

        let mut buf = self.buffer.lock();
        if buf.len() >= self.capacity {
            buf.pop_front();
        }
        buf.push_back(entry);
        drop(buf);

        // Notify subscribers
        for cb in self.subscribers.lock().iter() {
            cb();
        }
    }

    pub fn debug(&self, scope: &'static str, msg: impl Into<String>) { self.log(LogLevel::Debug, scope, msg); }
    pub fn info(&self, scope: &'static str, msg: impl Into<String>) { self.log(LogLevel::Info, scope, msg); }
    pub fn warn(&self, scope: &'static str, msg: impl Into<String>) { self.log(LogLevel::Warn, scope, msg); }
    pub fn error(&self, scope: &'static str, msg: impl Into<String>) { self.log(LogLevel::Error, scope, msg); }

    pub fn entries(&self) -> Vec<LogEntry> {
        self.buffer.lock().iter().cloned().collect()
    }

    pub fn clear(&self) {
        self.buffer.lock().clear();
    }
}

fn instant_now() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Convenience macros
#[macro_export]
macro_rules! log_debug { ($scope:expr, $($arg:tt)*) => { $crate::logger::logger().debug($scope, format!($($arg)*)) } }
#[macro_export]
macro_rules! log_info { ($scope:expr, $($arg:tt)*) => { $crate::logger::logger().info($scope, format!($($arg)*)) } }
#[macro_export]
macro_rules! log_warn { ($scope:expr, $($arg:tt)*) => { $crate::logger::logger().warn($scope, format!($($arg)*)) } }
#[macro_export]
macro_rules! log_error { ($scope:expr, $($arg:tt)*) => { $crate::logger::logger().error($scope, format!($($arg)*)) } }
