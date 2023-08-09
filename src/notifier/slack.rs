use serde::Serialize;
use serde_json::json;
use tokio::sync::broadcast;
use tracing::{debug, error};

use super::notifier::{Notifier, NotifierLogLevel};

use crate::watcher;

pub struct SlackNotifier {
    rx: broadcast::Receiver<watcher::Notification>,
    api_token: String,
    channel_id: String,
    client: reqwest::Client,
    log_level: NotifierLogLevel,
}

impl SlackNotifier {
    pub fn new(
        rx: broadcast::Receiver<watcher::Notification>,
        api_token: String,
        channel_id: String,
        log_level: NotifierLogLevel,
    ) -> Self {
        let client = reqwest::Client::new();

        Self {
            rx,
            api_token,
            channel_id,
            client,
            log_level,
        }
    }

    fn get_notification_color(level: NotifierLogLevel) -> &'static str {
        match level {
            NotifierLogLevel::Info => "#3498DB",
            NotifierLogLevel::Warn => "#FFC107",
            NotifierLogLevel::Error => "#FF0000",
        }
    }

    async fn send_notification(
        &self,
        watcher_notification: &watcher::Notification,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let color = Self::get_notification_color(watcher_notification.level().into());
        let message = watcher_notification.message();

        self.client
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(&self.api_token)
            .json(&json!({
                "channel": &self.channel_id,
                "attachments": [
                    {
                        "color": color,
                        "title": message
                    }
                ]
            }))
            .send()
            .await
    }
}

impl Notifier for SlackNotifier {
    fn notify(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Ok(notification) = self.rx.recv().await {
                let level: NotifierLogLevel = notification.level().into();
                if level < self.log_level {
                    debug!("Skipping sending notification. Notification log level ({}) is less than notifier log level ({})", level, self.log_level);
                    continue;
                }

                if let Err(e) = self.send_notification(&notification).await {
                    error!("Failed to send slack notification. Error: {:?}", e);
                }
            }
        })
    }
}

#[derive(Serialize, Clone)]
struct SlackNotification<'a> {
    pub(self) channel: &'a str,
    pub(self) title: &'a str,
    pub(self) text: &'a str,
    pub(self) color: &'a str,
}
