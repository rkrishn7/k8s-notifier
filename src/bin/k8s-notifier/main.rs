use clap::{ArgGroup, Parser};
use kube::Client;

use k8s_notifier::namespace::NamespaceScope;
use k8s_notifier::notifier::slack::SlackNotifier;
use k8s_notifier::notifier::{Notifier, NotifierLogLevel};
use k8s_notifier::resource::WatchedResource;
use k8s_notifier::ResourceWatcher;

/// A cluster utility that watches objects based on registered interest
/// and emits notifications of their status changes on external mediums
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(
    author = "Rohan Krishnaswamy <rohan@fastmail.us>",
    group(
        ArgGroup::new("namespace_scope").required(true).args(&["namespaces", "all_namespaces"]),
    ),
    group(
        ArgGroup::new("resource").required(true).args(&["resources"])
    )
)]
struct CliArgs {
    /// Resources to monitor
    #[arg(long, short, num_args = 1.., value_delimiter = ' ', env)]
    resources: Vec<WatchedResource>,
    /// Namespaces in which non cluster-scoped resources should be monitored
    #[arg(long, short, num_args = 1.., value_delimiter = ' ', env)]
    namespaces: Option<Vec<String>>,
    /// Watch resources in all namespaces
    #[arg(long, env)]
    all_namespaces: bool,
    /// Slack API token
    #[arg(long, env)]
    slack_token: String,
    /// Slack API token
    #[arg(long, env)]
    slack_channel: String,
    /// Log level for all notifiers
    #[arg(long, env, default_value_t = NotifierLogLevel::Error)]
    notifier_log_level: NotifierLogLevel,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    tracing_subscriber::fmt::init();

    let client = Client::try_default().await?;

    let ns_scope = if args.all_namespaces {
        NamespaceScope::All
    } else {
        NamespaceScope::Names(args.namespaces.unwrap_or_default())
    };

    let watcher = ResourceWatcher::new(client, ns_scope, args.resources);
    let mut handles = vec![];

    let (handle, rx) = watcher.watch();
    handles.push(handle);

    let notifier = SlackNotifier::new(
        rx,
        args.slack_token,
        args.slack_channel,
        args.notifier_log_level,
    );
    let handle = notifier.notify();
    handles.push(handle);

    for handle in handles {
        handle.await?;
    }

    Ok(())
}
