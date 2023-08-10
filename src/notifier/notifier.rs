use async_trait::async_trait;
use clap::ValueEnum;
use futures::future;
use futures::stream::StreamExt;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

use crate::resource::PackedResource;

#[derive(Debug, Clone, ValueEnum)]
pub enum NotifierType {
    Log,
    Slack,
}

impl std::fmt::Display for NotifierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotifierType::Log => write!(f, "log"),
            NotifierType::Slack => write!(f, "slack"),
        }
    }
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

/// A type that can be logged
pub trait Loggable {
    fn log_level(&self) -> NotifierLogLevel;
}

/// A notifier that outputs messages on a channel
#[async_trait]
pub trait Notifier:
    futures_core::Stream<Item = Result<PackedResource, BroadcastStreamRecvError>>
{
    type Notification: Send + Loggable;

    /// Creates zero or more notifications based on a Kubernetes resource
    fn create_notifications(&self, resource: &PackedResource) -> Vec<Self::Notification>;

    /// Configured log level for this notifier
    fn log_level(&self) -> NotifierLogLevel;

    /// Emits a notification on this notifier's channel
    async fn emit_notification(&self, notification: Self::Notification) -> anyhow::Result<()>;

    /// Runs this notifier
    fn run(mut self) -> tokio::task::JoinHandle<()>
    where
        Self: Sized + Unpin + Send + Sync + 'static,
    {
        tokio::spawn(async move {
            while let Some(resource) = self.next().await {
                if let Err(e) = resource {
                    tracing::error!(
                        "Notifier failed to read from resource broadcast stream. Error: {:?}",
                        e
                    );
                    continue;
                }

                let notifications =
                    self.create_notifications(&resource.expect("resource should be valid"));

                let results = future::join_all(
                    notifications
                        .into_iter()
                        .filter(|n| n.log_level() >= self.log_level())
                        .map(|notification| self.emit_notification(notification)),
                )
                .await;

                for result in results {
                    if let Err(e) = result {
                        tracing::error!("Notifier failed to send notification. Error: {:?}", e);
                    }
                }
            }
        })
    }
}

/// Implements [`futures_core::Stream<Item = Result<PackedResource, BroadcastStreamRecvError>>`]
/// for the specified type. Requires that the second argument is a [`tokio_stream::wrappers::BroadcastStream`]
macro_rules! impl_packed_resource_stream {
    ($name:ident, $prop:ident) => {
        impl futures_core::Stream for $name {
            type Item = Result<PackedResource, BroadcastStreamRecvError>;

            fn poll_next(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Option<Self::Item>> {
                self.$prop.poll_next_unpin(cx)
            }
        }
    };
}

/// Implements [`Loggable`] for the specified type
macro_rules! impl_loggable {
    ($name:ident, $prop:ident) => {
        impl Loggable for $name {
            fn log_level(&self) -> NotifierLogLevel {
                self.$prop
            }
        }
    };
}

pub(crate) use impl_loggable;
pub(crate) use impl_packed_resource_stream;
