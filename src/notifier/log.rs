use async_trait::async_trait;
use futures::StreamExt;
use tokio::sync::broadcast;
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

use super::{impl_loggable, impl_packed_resource_stream, Loggable, Notifier, NotifierLogLevel};

use crate::resource::PackedResource;

pub struct LogNotifier {
    rx: BroadcastStream<PackedResource>,
    log_level: NotifierLogLevel,
}

impl LogNotifier {
    pub fn new(rx: broadcast::Receiver<PackedResource>, log_level: NotifierLogLevel) -> Self {
        Self {
            rx: BroadcastStream::new(rx),
            log_level,
        }
    }
}

#[async_trait]
impl Notifier for LogNotifier {
    type Notification = LogNotification;

    fn create_notifications(&self, resource: &PackedResource) -> Vec<Self::Notification> {
        let mut notifications = vec![];

        match resource {
            PackedResource::Node(_) => notifications.push(LogNotification {
                level: NotifierLogLevel::Info,
                message: "Node seen".to_string(),
            }),
            PackedResource::Pod(_) => notifications.push(LogNotification {
                level: NotifierLogLevel::Info,
                message: "Pod seen".to_string(),
            }),
        }

        notifications
    }

    fn log_level(&self) -> NotifierLogLevel {
        self.log_level
    }

    async fn emit_notification(&self, notification: Self::Notification) -> anyhow::Result<()> {
        match notification.level {
            NotifierLogLevel::Info => {
                tracing::info!("{}", notification.message);
            }
            NotifierLogLevel::Warn => {
                tracing::warn!("{}", notification.message);
            }
            NotifierLogLevel::Error => {
                tracing::error!("{}", notification.message);
            }
        }

        Ok(())
    }
}

impl_packed_resource_stream!(LogNotifier, rx);

pub struct LogNotification {
    level: NotifierLogLevel,
    message: String,
}

impl_loggable!(LogNotification, level);
