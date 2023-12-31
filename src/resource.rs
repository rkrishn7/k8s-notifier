use clap::ValueEnum;
use k8s_openapi::api::core::v1::{Event, Node, Pod};

pub mod ext;

/// Packs API resources into a single type in order to create a unified resource
/// stream containing any registered resources for watching
#[derive(Debug, Clone)]
pub enum PackedResource {
    /// A Node resource
    Node(Node),
    /// A Pod resource
    Pod(Pod),
    /// A Event resource
    Event(Event),
}

/// A watched resource
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum WatchedResource {
    Node,
    Pod,
    Event,
}

impl std::fmt::Display for WatchedResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let typ = match self {
            WatchedResource::Node => "node",
            WatchedResource::Pod => "pod",
            WatchedResource::Event => "event",
        };

        write!(f, "{}", typ)
    }
}
