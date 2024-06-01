mod event;

use crate::event::EventExt;
use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use dapnet_api::{Client as DapnetClient, OutgoingCallBuilder};
use emfcamp_schedule_api::{
    announcer::{Announcer, AnnouncerPollResult, AnnouncerSettingsBuilder},
    Client as ScheduleClient,
};
use metrics::{counter, describe_counter};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use tokio::time::Duration as TokioDuration;
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

    /// Do not send notifications for events but log what would be sent (the start up check page is still sent)
    #[arg(long, env, default_value = "false")]
    dry_run: bool,

    /// Address on which to run the metrics endpoint
    #[arg(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    #[clap(subcommand)]
    mode: Mode,
}

#[derive(Debug, Subcommand)]
enum Mode {
    /// Send news to a single rubric
    Rubric {
        #[arg(long, env, default_value = "emfcamp")]
        rubric: String,
    },
    /// Send calls/pages to a set of individual recipients
    Call {
        #[arg(short, long = "recipient", env, value_name = "RECIPIENT")]
        recipients: Vec<String>,
    },
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
    send_startup_page(&dapnet, &cli.dapnet_username).await?;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                return Ok(());
            }
            msg = announcer.poll() => {
                handle_announcer_event(&dapnet, cli.dry_run, &cli.mode, msg).await;
            }
        }
    }
}

async fn handle_announcer_event(
    dapnet: &DapnetClient,
    dry_run: bool,
    mode: &Mode,
    msg: emfcamp_schedule_api::Result<AnnouncerPollResult>,
) {
    match msg {
        Ok(AnnouncerPollResult::Event(event)) => match mode {
            Mode::Rubric { rubric } => {
                if let Some(news) = event.to_rubric_news(rubric.clone()) {
                    info!("News for event: {:?}", news);

                    if !dry_run {
                        for attempt in 1..6 {
                            info!("Trying to send news... (attempt {attempt})");
                            match dapnet.new_news(&news).await {
                                Ok(_) => {
                                    info!("News sent");
                                    counter!("dapnet_event_announcements", "target" => "rubric", "result" => "ok").increment(1);
                                    break;
                                }
                                Err(e) => {
                                    error!("Failed to send news: {e}");
                                    counter!("dapnet_event_announcements", "target" => "rubric", "result" => "error").increment(1);
                                    tokio::time::sleep(TokioDuration::from_secs(1)).await;
                                }
                            }
                        }
                    }
                }
            }
            Mode::Call { recipients } => {
                if let Some(call) = event.to_call(recipients.clone()) {
                    info!("Call for event: {:?}", call);

                    if !dry_run {
                        for attempt in 1..6 {
                            info!("Trying to send news... (attempt {attempt})");
                            match dapnet.new_call(&call).await {
                                Ok(_) => {
                                    info!("Call sent");
                                    counter!("dapnet_event_announcements", "target" => "call", "result" => "ok").increment(1);
                                    break;
                                }
                                Err(e) => {
                                    error!("Failed to send call: {e}");
                                    counter!("dapnet_event_announcements", "target" => "call", "result" => "error").increment(1);
                                    tokio::time::sleep(TokioDuration::from_secs(1)).await;
                                }
                            }
                        }
                    }
                }
            }
        },
        Err(e) => {
            warn!("{e}");
        }
        _ => {}
    }
}

async fn send_startup_page(dapnet: &DapnetClient, recipient: &str) -> anyhow::Result<()> {
    info!("Checking DAPNET connection...");

    match dapnet
        .new_call(
            &OutgoingCallBuilder::default()
                .text(format!(
                    "{recipient}: EMF sched. anncr. start at {}",
                    Utc::now().format("%d %H:%M %Z")
                ))
                .recipients(vec![recipient.to_string()])
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
