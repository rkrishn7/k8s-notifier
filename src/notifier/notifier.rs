use clap::ValueEnum;

use crate::watcher;

pub trait Notifier {
    /// Begins emitting notifications
    fn notify(self) -> tokio::task::JoinHandle<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, ValueEnum)]
pub enum NotifierLogLevel {
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for NotifierLogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotifierLogLevel::Info => write!(f, "info"),
            NotifierLogLevel::Warn => write!(f, "warn"),
            NotifierLogLevel::Error => write!(f, "error"),
        }
    }
}

impl From<watcher::NotificationLogLevel> for NotifierLogLevel {
    fn from(value: watcher::NotificationLogLevel) -> Self {
        match value {
            watcher::NotificationLogLevel::Error => Self::Error,
            watcher::NotificationLogLevel::Warn => Self::Warn,
            watcher::NotificationLogLevel::Info => Self::Info,
        }
    }
}
