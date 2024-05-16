mod event_news;

use crate::event_news::EventExt;
use chrono::{Duration, Utc};
use clap::Parser;
use dapnet_api::{Client as DapnetClient, OutgoingCallBuilder};
use emfcamp_schedule_api::{
    announcer::{Announcer, AnnouncerPollResult, AnnouncerSettingsBuilder},
    Client as ScheduleClient,
};
use metrics::{counter, describe_counter};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use tracing::{error, info, warn};
use url::Url;

/// Announces the EMF schedule via DAPNET
#[derive(Debug, Parser)]
struct Cli {
    /// Address of schedule API to source event data from
    #[arg(
        long,
        env,
        default_value = "https://schedule.emfcamp.dan-nixon.com/schedule"
    )]
    api_url: Url,

    /// DAPNET username (user must have access to the emfcamp rubric)
    #[arg(long, env)]
    dapnet_username: String,

    /// DAPNET password
    #[arg(long, env)]
    dapnet_password: String,

    /// Time in seconds before the start time of an event to send the notification
    #[arg(long, env, default_value = "120")]
    pre_event_announcement_time: i64,

    /// Do not send notifications for events (the start up check page is still sent)
    #[arg(long, env, default_value = "false")]
    dry_run: bool,

    /// Address on which to run the metrics endpoint
    #[arg(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt::init();

    // Set up metrics server
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(cli.observability_address)
        .install()?;

    describe_counter!(
        "dapnet_event_announcements",
        "Number of announcements sent to DAPNET"
    );

    // Setup schedule API client
    let schedule_client = ScheduleClient::new(cli.api_url);

    let event_start_offset = -Duration::try_seconds(cli.pre_event_announcement_time)
        .ok_or_else(|| anyhow::anyhow!("Invalid pre event announcement time"))?;
    info!("Event start offset: {:?}", event_start_offset);

    let mut announcer = Announcer::new(
        AnnouncerSettingsBuilder::default()
            .event_start_offset(event_start_offset)
            .build()?,
        schedule_client,
    )
    .await?;

    // Setup and test DAPNET client
    let dapnet = DapnetClient::new(&cli.dapnet_username, &cli.dapnet_password);
    send_startup_page(&dapnet).await?;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                return Ok(());
            }
            msg = announcer.poll() => {
                handle_announcer_event(&dapnet, cli.dry_run, msg).await;
            }
        }
    }
}

async fn handle_announcer_event(
    dapnet: &DapnetClient,
    dry_run: bool,
    msg: emfcamp_schedule_api::Result<AnnouncerPollResult>,
) {
    match msg {
        Ok(AnnouncerPollResult::Event(event)) => {
            if let Some(news) = event.to_rubric_news() {
                info!("News for event: {:?}", news);

                if !dry_run {
                    match dapnet.new_news(&news).await {
                        Ok(_) => {
                            info!("News sent");
                            counter!("dapnet_event_announcements", "result" => "ok").increment(1);
                        }
                        Err(e) => {
                            error!("Failed to send news: {e}");
                            counter!("dapnet_event_announcements", "result" => "error")
                                .increment(1);
                        }
                    }
                }
            }
        }
        Err(e) => {
            warn!("{e}");
        }
        _ => {}
    }
}

async fn send_startup_page(dapnet: &DapnetClient) -> anyhow::Result<()> {
    info!("Checking DAPNET connection...");

    match dapnet
        .new_call(
            &OutgoingCallBuilder::default()
                .text(format!(
                    "M0NXN: EMF sched. anncr. start at {}",
                    Utc::now().format("%d %H:%M %Z")
                ))
                .recipients(vec!["m0nxn".to_string()])
                .transmitter_groups(vec!["uk-all".to_string()])
                .build()?,
        )
        .await
    {
        Ok(()) => {
            info!("Could send a page, assuming DAPNET connection is working");
        }
        Err(e) => {
            warn!("Failed to send a page, something's fucky... {e}");
        }
    };

    Ok(())
}
