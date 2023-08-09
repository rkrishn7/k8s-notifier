#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLogLevel,
}

impl Notification {
    pub fn level(&self) -> NotificationLogLevel {
        self.level
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationLogLevel {
    Error,
    Warn,
    Info,
}
