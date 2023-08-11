use async_trait::async_trait;
use futures::StreamExt;
use serde_json::json;
use tokio::sync::broadcast;
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

use super::{impl_loggable, impl_packed_resource_stream, Loggable, Notifier, NotifierLogLevel};

use crate::resource::ext::event::EventExt;
use crate::resource::ext::node::NodeExt;
use crate::resource::ext::pod::PodExt;
use crate::resource::PackedResource;

pub struct SlackNotifier {
    rx: BroadcastStream<PackedResource>,
    api_token: String,
    channel_id: String,
    client: reqwest::Client,
    log_level: NotifierLogLevel,
    cluster_name: String,
}

impl SlackNotifier {
    pub fn new(
        rx: broadcast::Receiver<PackedResource>,
        api_token: String,
        channel_id: String,
        log_level: NotifierLogLevel,
        cluster_name: String,
    ) -> Self {
        let client = reqwest::Client::new();

        Self {
            rx: BroadcastStream::new(rx),
            api_token,
            channel_id,
            client,
            log_level,
            cluster_name,
        }
    }
}

#[async_trait]
impl Notifier for SlackNotifier {
    type Notification = SlackNotification;

    // TODO(rkrishn7): Refactor this monstrosity
    fn create_notifications(&self, resource: &PackedResource) -> Vec<Self::Notification> {
        let mut notifications = vec![];

        match resource {
            PackedResource::Node(node) => {
                let labels_formatted = node
                    .labels()
                    .iter()
                    .map(|(lbl, val)| format!("• `{lbl}` : `{val}`"))
                    .collect::<Vec<String>>()
                    .join("\n");

                let addresses_formatted = node
                    .addresses()
                    .unwrap_or_default()
                    .iter()
                    .map(|(typ, addr)| format!("• `{typ}` : `{addr}`"))
                    .collect::<Vec<String>>()
                    .join("\n");

                let conditions_formatted = node
                    .status_conditions()
                    .unwrap_or_default()
                    .iter()
                    .map(|(cond, val)| format!("• `{cond}` : `{val}`"))
                    .collect::<Vec<String>>()
                    .join("\n");

                let labels_section = format!(
                    "*Labels*\n{}",
                    if labels_formatted.is_empty() {
                        "<None>".to_string()
                    } else {
                        labels_formatted
                    }
                );
                let addresses_section = format!(
                    "*Addresses*\n{}",
                    if addresses_formatted.is_empty() {
                        "<None>".to_string()
                    } else {
                        addresses_formatted
                    }
                );
                let conditions_section = format!(
                    "*Conditions*\n{}",
                    if conditions_formatted.is_empty() {
                        "<None>".to_string()
                    } else {
                        conditions_formatted
                    }
                );

                let (title, log_level) = if node.unschedulable() {
                    (
                        format!(
                            "Node `{}` is *unschedulable* in cluster `{}`",
                            node.name(),
                            self.cluster_name
                        ),
                        NotifierLogLevel::Error,
                    )
                } else {
                    (
                        format!(
                            "Node `{}` is *healthy* in cluster `{}`",
                            node.name(),
                            self.cluster_name
                        ),
                        NotifierLogLevel::Info,
                    )
                };
                let color = get_notification_color(log_level);

                notifications.push(SlackNotification {
                    inner: json!(
                        {
                        "channel": &self.channel_id,
                        "attachments": [
                            {
                                "color": color,
                                "blocks": [
                                    {
                                        "type": "section",
                                        "text": {
                                            "type": "mrkdwn",
                                            "text": title,
                                        }
                                    },
                                    {
                                        "type": "divider"
                                    },
                                    {
                                        "type": "section",
                                        "fields": [
                                            {
                                                "type": "mrkdwn",
                                                "text": conditions_section
                                            },
                                            {
                                                "type": "mrkdwn",
                                                "text": addresses_section
                                            },
                                        ]
                                    },
                                    {
                                        "type": "section",
                                        "fields": [
                                            {
                                                "type": "mrkdwn",
                                                "text": labels_section
                                            },
                                        ]
                                    },
                                ]
                            }
                        ]
                    }),
                    level: log_level,
                });
            }
            PackedResource::Pod(pod) => {
                if let Some(phase) = pod.phase() {
                    let labels_formatted = pod
                        .labels()
                        .iter()
                        .map(|(lbl, val)| format!("• `{lbl}` : `{val}`"))
                        .collect::<Vec<String>>()
                        .join("\n");

                    let labels_section = format!(
                        "*Labels*\n{}",
                        if labels_formatted.is_empty() {
                            "<None>".to_string()
                        } else {
                            labels_formatted
                        }
                    );

                    let name = pod.name();
                    let namespace_section = format!(
                        "*Namespace*\n`{}`",
                        pod.namespace().unwrap_or("<Unknown>".to_string())
                    );
                    let ip_section = format!(
                        "*IP Address*\n`{}`",
                        pod.ip_addr().unwrap_or(&"<None>".to_string())
                    );
                    let title = format!(
                        "Pod `{name}` is in phase *{phase}* in cluster `{}`",
                        self.cluster_name
                    );

                    let log_level = if phase == "Running" || phase == "Succeeded" {
                        NotifierLogLevel::Info
                    } else if phase == "Pending" {
                        NotifierLogLevel::Warn
                    } else {
                        NotifierLogLevel::Error
                    };
                    let color = get_notification_color(log_level);

                    notifications.push(SlackNotification {
                        inner: json!(
                            {
                            "channel": &self.channel_id,
                            "attachments": [
                                {
                                    "color": color,
                                    "blocks": [
                                        {
                                            "type": "section",
                                            "text": {
                                                "type": "mrkdwn",
                                                "text": title,
                                            }
                                        },
                                        {
                                            "type": "divider"
                                        },
                                        {
                                            "type": "section",
                                            "fields": [
                                                {
                                                    "type": "mrkdwn",
                                                    "text": namespace_section
                                                },
                                                {
                                                    "type": "mrkdwn",
                                                    "text": ip_section
                                                },
                                            ]
                                        },
                                        {
                                            "type": "section",
                                            "fields": [
                                                {
                                                    "type": "mrkdwn",
                                                    "text": labels_section
                                                },
                                            ]
                                        },
                                    ]
                                }
                            ]
                        }),
                        level: log_level,
                    });
                }
            }
            PackedResource::Event(event) => {
                if let Some(typ) = event.typ() {
                    let log_level = if typ == "Normal" {
                        NotifierLogLevel::Info
                    } else {
                        NotifierLogLevel::Warn
                    };
                    let color = get_notification_color(log_level);
                    let title = format!(
                        "*{}* events seen from the {} `{}`",
                        event.count().unwrap_or(0),
                        event
                            .involved_object_kind()
                            .unwrap_or(&"<Unknown Resource>".to_string()),
                        event
                            .involved_object_name()
                            .unwrap_or(&"<Unknown Name>".to_string())
                    );

                    let first_seen_section = format!(
                        "*First Seen*\n`{}`",
                        event
                            .first_timestamp()
                            .map(|t| t.to_string())
                            .unwrap_or("<Unknown>".to_string())
                    );
                    let last_seen_section = format!(
                        "*Last Seen*\n`{}`",
                        event
                            .last_timestamp()
                            .map(|t| t.to_string())
                            .unwrap_or("<Unknown>".to_string())
                    );
                    let message_section = format!(
                        "*Message*\n{}",
                        event.message().unwrap_or(&"\"\"".to_string())
                    );
                    let reason_section = format!(
                        "*Reason*\n{}",
                        event.reason().unwrap_or(&"\"\"".to_string())
                    );

                    notifications.push(SlackNotification {
                        inner: json!(
                            {
                            "channel": &self.channel_id,
                            "attachments": [
                                {
                                    "color": color,
                                    "blocks": [
                                        {
                                            "type": "section",
                                            "text": {
                                                "type": "mrkdwn",
                                                "text": title,
                                            }
                                        },
                                        {
                                            "type": "divider"
                                        },
                                        {
                                            "type": "section",
                                            "fields": [
                                                {
                                                    "type": "mrkdwn",
                                                    "text": first_seen_section
                                                },
                                                {
                                                    "type": "mrkdwn",
                                                    "text": last_seen_section
                                                },
                                            ]
                                        },
                                        {
                                            "type": "section",
                                            "fields": [
                                                {
                                                    "type": "mrkdwn",
                                                    "text": reason_section
                                                },
                                                {
                                                    "type": "mrkdwn",
                                                    "text": message_section
                                                },
                                            ]
                                        },
                                    ]
                                }
                            ]
                        }),
                        level: log_level,
                    });
                }
            }
        }

        notifications
    }

    fn log_level(&self) -> NotifierLogLevel {
        self.log_level
    }

    async fn emit_notification(&self, notification: Self::Notification) -> anyhow::Result<()> {
        let res = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(&self.api_token)
            .json(&notification.inner)
            .send()
            .await?;

        tracing::info!(
            "Received status {} upon emitting slack notification",
            res.status()
        );

        Ok(())
    }
}

impl_packed_resource_stream!(SlackNotifier, rx);

pub struct SlackNotification {
    level: NotifierLogLevel,
    inner: serde_json::Value,
}

impl_loggable!(SlackNotification, level);

fn get_notification_color(level: NotifierLogLevel) -> &'static str {
    match level {
        NotifierLogLevel::Info => "#3498DB",
        NotifierLogLevel::Warn => "#FFC107",
        NotifierLogLevel::Error => "#FF0000",
    }
}
