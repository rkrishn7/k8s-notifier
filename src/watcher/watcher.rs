use std::pin::Pin;

use futures::{Stream, StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::{Node, Pod};
use kube::{
    api::Api,
    runtime::{watcher, WatchStreamExt},
    Client, ResourceExt,
};
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::{debug, error};

use super::notification::{Notification, NotificationLogLevel};
use crate::namespace::NamespaceScope;
use crate::resource::{PackedResource, WatchedResource};

pub struct ResourceWatcher {
    client: Client,
    namespace_scope: NamespaceScope,
    resources: Vec<WatchedResource>,
}

impl ResourceWatcher {
    pub fn new(
        client: Client,
        namespace_scope: NamespaceScope,
        resources: Vec<WatchedResource>,
    ) -> Self {
        Self {
            client,
            namespace_scope,
            resources,
        }
    }

    fn create_multiplexed_resource_stream(
        &self,
    ) -> futures::stream::SelectAll<
        Pin<
            Box<
                dyn Stream<Item = Result<PackedResource, kube::runtime::watcher::Error>>
                    + std::marker::Send,
            >,
        >,
    > {
        let mut streams = vec![];
        for resource in &self.resources {
            match resource {
                WatchedResource::Node => {
                    let nodes: Api<Node> = Api::all(self.client.clone());
                    streams.push(
                        watcher(nodes, watcher::Config::default())
                            .default_backoff()
                            .applied_objects()
                            .map_ok(PackedResource::Node)
                            .boxed(),
                    );
                }
                WatchedResource::Pod => match &self.namespace_scope {
                    NamespaceScope::All => {
                        let pods: Api<Pod> = Api::all(self.client.clone());
                        streams.push(
                            watcher(pods, watcher::Config::default())
                                .default_backoff()
                                .applied_objects()
                                .map_ok(PackedResource::Pod)
                                .boxed(),
                        )
                    }
                    NamespaceScope::Names(names) => {
                        streams.extend(names.iter().map(|name| {
                            let pods: Api<Pod> =
                                Api::namespaced(self.client.clone(), name.as_str());

                            watcher(pods, watcher::Config::default())
                                .default_backoff()
                                .applied_objects()
                                .map_ok(PackedResource::Pod)
                                .boxed()
                        }));
                    }
                },
            }
        }

        futures::stream::select_all(streams)
    }

    pub fn watch(&self) -> (JoinHandle<()>, broadcast::Receiver<Notification>) {
        let mut stream = self.create_multiplexed_resource_stream();
        let (tx, rx) = broadcast::channel(256);

        let handle = tokio::spawn(async move {
            while let Some(resource) = stream.next().await {
                match resource {
                    Ok(resource) => {
                        let result = match resource {
                            PackedResource::Node(node) => {
                                Self::broadcast_node_notifications(&node, &tx)
                            }
                            PackedResource::Pod(_) => todo!(),
                        };

                        if let Err(e) = result {
                            error!("Error broadcasting resource update {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Received error while reading from resource stream {:?}", e);
                    }
                }
            }
        });

        (handle, rx)
    }

    fn broadcast_node_notifications(
        node: &Node,
        tx: &broadcast::Sender<Notification>,
    ) -> anyhow::Result<()> {
        let node_name = node.name_any();

        let notification = if let Some(true) = node
            .spec
            .as_ref()
            .expect("node should have spec")
            .unschedulable
        {
            Notification {
                message: format!("Node {} is unschedulable", node_name),
                level: NotificationLogLevel::Error,
            }
        } else {
            Notification {
                message: format!("Node {} is healthy", node_name),
                level: NotificationLogLevel::Info,
            }
        };

        let num_notifiers = tx.send(notification)?;

        debug!("Sent notification to {num_notifiers} notifiers");

        Ok(())
    }
}
