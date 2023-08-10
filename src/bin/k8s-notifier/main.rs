use clap::{ArgGroup, Parser};
use kube::Client;

use k8s_notifier::namespace::NamespaceScope;
use k8s_notifier::notifier::log::LogNotifier;
use k8s_notifier::notifier::slack::SlackNotifier;
use k8s_notifier::notifier::{Notifier, NotifierLogLevel, NotifierType};
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
    /// The notifiers to run
    #[arg(long, num_args = 1.., value_delimiter = ' ', env)]
    notifiers: Vec<NotifierType>,
    /// Watch resources in all namespaces
    #[arg(long, env)]
    all_namespaces: bool,
    /// Slack API token. Required if 'slack' is configured as a notifier
    #[arg(long, env)]
    slack_token: Option<String>,
    /// Slack API token. Required if 'slack' is configured as a notifier
    #[arg(long, env)]
    slack_channel: Option<String>,
    /// Log level for all notifiers
    #[arg(long, env, default_value_t = NotifierLogLevel::Error)]
    notifier_log_level: NotifierLogLevel,
    /// The name of the Kubernetes cluster we're running in. Used for logging
    /// purposes
    #[arg(long, env)]
    cluster_name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = CliArgs::parse();
    tracing_subscriber::fmt::init();

    let client = Client::try_default().await?;

    let ns_scope = if args.all_namespaces {
        NamespaceScope::All
    } else {
        NamespaceScope::Names(args.namespaces.unwrap_or_default())
    };

    let mut handles = vec![];
    let watcher = ResourceWatcher::new(client, ns_scope, args.resources);

    let (handle, tx) = watcher.watch();
    handles.push(handle);

    for notifier in args.notifiers {
        let handle = match notifier {
            NotifierType::Log => {
                let log_notifier = LogNotifier::new(tx.subscribe(), args.notifier_log_level);

                log_notifier.run()
            }
            NotifierType::Slack => {
                let slack_notifier = SlackNotifier::new(
                    tx.subscribe(),
                    args.slack_token.take().expect("SLACK_TOKEN/--slack-token must be set if the 'slack' notifier is enabled"),
                    args.slack_channel.take().expect("SLACK_CHANNEL/--slack-channel must be set if the 'slack' notifier is enabled"),
                    args.notifier_log_level,
                    args.cluster_name.clone(),
                );

                slack_notifier.run()
            }
        };

        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}
