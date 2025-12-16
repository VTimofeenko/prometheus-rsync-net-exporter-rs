mod config;
mod parser;
mod ssh;

use crate::config::Config;
use crate::parser::parse_quota_output;
use crate::ssh::{RsyncFetcher, SshFetcher};
use anyhow::Result;
use metrics::{describe_gauge, gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

#[tokio::main]
async fn main() -> Result<()> {
    setup_logging();

    let config = Config::load().map_err(|e| anyhow::anyhow!(e))?;
    info!("Configuration loaded for user: {}", config.username);

    let builder = PrometheusBuilder::new();
    info!("Listening on {}:{}", config.listen_address, config.port);
    builder
        .with_http_listener((config.listen_address, config.port))
        .install()
        .expect("failed to install Prometheus recorder");

    register_metrics();

    let fetcher = Arc::new(SshFetcher::new(
        config.username.clone(),
        config.host.clone(),
        config.ssh_key_path.clone(),
    ));

    let fetch_interval = Duration::from_secs(config.fetch_interval_seconds);
    let mut interval = time::interval(fetch_interval);

    info!("Starting metrics loop with interval: {:?}", fetch_interval);

    loop {
        interval.tick().await;
        info!("Fetching quota...");

        let fetcher_clone = fetcher.clone();

        let result = tokio::task::spawn_blocking(move || fetcher_clone.fetch_quota()).await?;

        match result {
            Ok(output) => {
                match parse_quota_output(&output) {
                    Ok(quotas) => {
                        gauge!("rsync_net_up").set(1.0);
                        if let Some(quota) = quotas.first() {
                            info!("Updated metrics for filesystem: {}", quota.filesystem);
                            gauge!("rsync_net_usage").set(quota.usage);
                            gauge!("rsync_net_soft_quota").set(quota.soft_quota);
                            gauge!("rsync_net_hard_quota").set(quota.hard_quota);
                            gauge!("rsync_net_files").set(quota.files as f64);
                            gauge!("rsync_net_billed_usage").set(quota.billed_usage);
                            gauge!("rsync_net_free_snaps").set(quota.free_snaps);
                            gauge!("rsync_net_custom_snaps").set(quota.custom_snaps);
                        } else {
                            warn!("No quota lines found in output");
                            // Should we set up=0 here? Maybe just warn.
                            gauge!("rsync_net_up").set(0.0);
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse quota output: {:#}", e);
                        gauge!("rsync_net_up").set(0.0);
                    }
                }
            }
            Err(e) => {
                error!("Failed to fetch quota via SSH: {:#}", e);
                gauge!("rsync_net_up").set(0.0);
            }
        }
    }
}

fn register_metrics() {
    describe_gauge!("rsync_net_up", "1 if fetch was OK, 0 if not OK");
    describe_gauge!(
        "rsync_net_usage",
        "Amount of space occupied by current backups (GB)"
    );
    describe_gauge!("rsync_net_soft_quota", "Soft quota (GB)");
    describe_gauge!("rsync_net_hard_quota", "Hard quota (GB)");
    describe_gauge!("rsync_net_files", "Count of files");
    describe_gauge!("rsync_net_billed_usage", "Disk space that is billed (GB)");
    describe_gauge!(
        "rsync_net_free_snaps",
        "Amount of space occupied by free snapshots (GB)"
    );
    describe_gauge!(
        "rsync_net_custom_snaps",
        "Amount of space occupied by custom snapshots (GB)"
    );
}

fn setup_logging() {
    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_span_events(FmtSpan::NEW)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}
