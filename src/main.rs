use std::num::NonZeroU64;
use std::sync::Arc;
use std::{fs::File, str::FromStr};

use anyhow::{Context, Result};
use cached::proc_macro::cached;
use chrono::Local;
use chrono_humanize::Humanize;
use cron::Schedule;
use log::{debug, error, info};
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};
use tokio::sync::mpsc::{self, Sender};
use tokio::time;
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_http::Client as HttpClient;

use aoc_bot::{
    aoc::{self, LeaderboardStats, User},
    discord,
    models::{Event, Message},
    settings::{Logging, Settings},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Loading .env file
    dotenv::dotenv().ok();
    // Load settings file
    let settings = Settings::new().await?;

    setup_logger(&settings.logging)?;

    info!("Starting ...");
    let (events_tx, mut events_rx) = mpsc::channel(1);
    discord::start(&settings.discord, events_tx.clone()).await?;

    if let Some(schedule) = settings.discord.schedule {
        debug!("Setting up scheduled leaderboard messages");

        let interval =
            Schedule::from_str(&schedule.interval).context("Invalid schedule interval")?;

        tokio::spawn(async move {
            if let Err(e) = run_scheduler(schedule.channel_id, interval, events_tx).await {
                error!("failed running scheduler: {:?}", e);
            }
        });
    }

    // HTTP is separate from the gateway, so create a new client.
    debug!("Setting up http client for twilight");

    let http = Arc::new(discord::new_client(settings.discord.bot_token));

    let board_id = Arc::from(settings.aoc.board_id);
    let session_cookie = Arc::from(settings.aoc.session_cookie);

    // Process each event as they come in.
    while let Some(event) = events_rx.recv().await {
        if matches!(event, Event::Shutdown) {
            break;
        }

        let fut = handle_event(
            event,
            Arc::clone(&http),
            Arc::clone(&board_id),
            Arc::clone(&session_cookie),
            settings.aoc.event_year,
        );

        tokio::spawn(async {
            if let Err(e) = fut.await {
                error!("failed handling event: {:?}", e);
            }
        });
    }

    Ok(())
}

#[cached(
    time = 7200,
    result = true,
    with_cached_flag = true,
    key = "String",
    convert = r#"{ format!("{}-{}", session_cookie, leaderboard_id) }"#
)]
async fn get_aoc_data(
    session_cookie: &str,
    leaderboard_id: &str,
    event: &u16,
) -> Result<cached::Return<LeaderboardStats>> {
    Ok(cached::Return::new(
        aoc::get_private_leaderboard_stats(session_cookie, event, leaderboard_id).await?,
    ))
}

async fn handle_event(
    event: Event,
    http: Arc<HttpClient>,
    board_id: Arc<str>,
    session_cookie: Arc<str>,
    event_year: u16,
) -> Result<()> {
    match event {
        Event::Ping(msg) => {
            info!("Ping message");
            let r = http
                .create_message(msg.channel_id.into())
                .content(":ping_pong: Pong! - Ping [000]ms")?
                .exec()
                .await?;
            let resmsg = r.model().await?;
            http.update_message(msg.channel_id.into(), resmsg.id)
                .content(Some(
                    format!(
                        ":ping_pong: Pong! - Ping [{:0>3}]ms",
                        (resmsg.timestamp.as_micros() - msg.timestamp.unwrap().as_micros()) / 1000
                    )
                    .as_str(),
                ))?
                .exec()
                .await?;
        }
        Event::AdventOfCode(msg) => {
            if let Some(author) = msg.author {
                info!(
                    "Request from ({}) {} to get aoc board",
                    author.id, author.name
                );
            } else {
                info!("Automated request");
            }

            let data = get_aoc_data(&session_cookie, &board_id, &event_year).await?;

            debug!(
                "Retrieved data (cached: {}) -> constructing message",
                data.was_cached
            );
            let mut embed = EmbedBuilder::new()
                .title(format!("AoC Leaderboard [{}]", board_id))
                .description(format!(
                    "Here is your current Leaderboard - Cached [{}]",
                    data.was_cached
                ));

            let mut uvec = data.members.values().collect::<Vec<_>>();
            uvec.sort_by(|a, b| b.local_score.cmp(&a.local_score));

            for (idx, user) in uvec.iter().enumerate() {
                embed = embed.field(
                    EmbedFieldBuilder::new(
                        format!("#{} - {} - {} score", idx + 1, user.name, user.local_score),
                        format!(
                            "⭐ Solved {} Challenges\n⏱️ Last at {}",
                            user.stars,
                            latest_challenge(user)
                        ),
                    )
                    .inline()
                    .build(),
                );
            }
            debug!("sending discord message to {}", msg.channel_id);
            http.create_message(msg.channel_id.into())
                .embeds(&[embed.build()?])?
                .exec()
                .await?;
        }
        Event::FourtyTwo(msg) => {
            info!("42 message");
            http.create_message(msg.channel_id.into())
                .content(
                    ":exploding_head: \
                    The Answer to the Ultimate Question of Life, \
                    the Universe, and Everything is 42",
                )?
                .exec()
                .await?;
        }
        Event::TopThree(msg) => {
            info!("getting top 3");

            let data = get_aoc_data(&session_cookie, &board_id, &event_year).await?;
            let mut uvec = data.members.values().collect::<Vec<_>>();

            if uvec.len() < 3 {
                http.create_message(msg.channel_id.into())
                    .content(":exclamation: Sorry, but there are not 3 people on your leaderboard, and you do not fill these 3 steps alone")?
                    .exec()
                    .await?;
                return Ok(());
            }

            uvec.sort_by_key(|m| m.local_score);

            debug!(
                "Retrieved data (cached: {}) -> constructing message",
                data.was_cached
            );

            let text = format!(
                "```\n
                {0:^15}
                  ↑ {1: ^3} points
                  ★ {2: ^3} stars
                 _____________
                /     ___     \\
                |    /   |    |
                |   /_   |    |
{3:^15} |     |  |    |    {6:^15}
↑ {4:^3} points    |     |  |    |    ↑ {7:^3} points
★ {5:^3} stars     |     |__|    |    ★ {8:^3} stars
   _____________|             |_____________
  /    _____                       _____    \\
  |   |__   |                     |__   |   |
  |    __|  |                      __|  |   |
  |   |   __|                     |__   |   |
  |   |  |__                       __|  |   |
  |   |_____|                     |_____|   |
  \\_________________________________________/ ```",
                uvec[0].name,
                uvec[0].local_score,
                uvec[0].stars,
                uvec[1].name,
                uvec[1].local_score,
                uvec[1].stars,
                uvec[2].name,
                uvec[2].local_score,
                uvec[2].stars
            );

            http.create_message(msg.channel_id.into())
                .content(&text)?
                .exec()
                .await?;
        }
        _ => {}
    }

    Ok(())
}

/// Get the latest completion time of the latest challenge from a single user. First check whether
/// part 1 or 2 was solved latest (as part 2 may not be solved yet) for each day and then compares
/// this timestamp with the other days.
fn latest_challenge(user: &User) -> String {
    let max = user
        .completion_day_level
        .values()
        .map(|day| {
            if let Some(part2) = &day.part2 {
                day.part1.get_star_ts.max(part2.get_star_ts)
            } else {
                day.part1.get_star_ts
            }
        })
        .max();

    match max {
        None => "...never".to_owned(),
        Some(ts) => ts.humanize(),
    }
}

/// Start up a fixed scheduler that periodically sends leaderboard statistics based on the
/// configured cron schedule.
async fn run_scheduler(
    channel_id: NonZeroU64,
    interval: Schedule,
    tx: Sender<Event>,
) -> Result<()> {
    let mut interval = interval.upcoming(Local);

    loop {
        let next = interval.next().context("no future scheduling event")?;
        let duration = (next - Local::now()).to_std()?;

        debug!(
            "Next scheduled dashboard message in {}",
            humantime::format_duration(duration)
        );

        time::sleep(duration).await;

        debug!("Sending new schedule event");
        let res = tx
            .send(Event::AdventOfCode(Message {
                channel_id,
                author: None,
                timestamp: None,
            }))
            .await;

        if let Err(e) = res {
            error!("failed sending scheduled leaderboard: {:?}", e);
        }
    }
}

/// Set up an combined logger which will log to the terminal and a file. Whether a logger is enabled
/// or what level it logs at is defined by the given configuration.
fn setup_logger(config: &Logging) -> Result<()> {
    let mut loggers = Vec::<Box<dyn SharedLogger>>::new();
    let log_config = ConfigBuilder::new().add_filter_allow_str("aoc_bot").build();

    if let Some(terminal) = &config.terminal {
        loggers.push(TermLogger::new(
            terminal.filter,
            log_config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ));
    };

    if let Some(file) = &config.file {
        loggers.push(WriteLogger::new(
            file.base.filter,
            log_config,
            File::create(&file.path)?,
        ));
    }

    CombinedLogger::init(loggers).context("logger failed to set up")
}
