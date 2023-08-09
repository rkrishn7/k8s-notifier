pub mod namespace;
pub mod notifier;
pub mod resource;
pub mod watcher;

pub use notifier::slack::SlackNotifier;
pub use watcher::ResourceWatcher;
