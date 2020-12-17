use std::fs::File;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use cached::proc_macro::cached;
use chrono_humanize::Humanize;
use log::{debug, info};
use simplelog::{
    CombinedLogger, ConfigBuilder, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};
use tokio::sync::mpsc;
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
    let (mut events_tx, mut events_rx) = mpsc::channel(1);
    discord::start(&settings.discord, events_tx.clone()).await?;

    if let Some(schedule) = &settings.discord.schedule {
        debug!("Setting up scheduled leaderboard messages");

        let interval = schedule.interval;
        let channel_id = schedule.channel_id;

        tokio::spawn(async move {
            let mut ticker = time::interval(Duration::from_secs(interval));
            // First tick completes immediately so we wait on the first tick here once to not
            // directly send statistics whenever the server starts up.
            ticker.tick().await;

            loop {
                ticker.tick().await;
                debug!("Sending new schedule event");

                let res = events_tx
                    .send(Event::AdventOfCode(Message {
                        shard_id: 0,
                        channel_id,
                        author: None,
                    }))
                    .await;

                if res.is_err() {
                    break;
                }
            }
        });
    }

    // HTTP is separate from the gateway, so create a new client.
    debug!("Setting up http client for twilight");
    let http = discord::new_client(&settings.discord);

    let board_id = Arc::new(settings.aoc.board_id.into_boxed_str());
    let session_cookie = Arc::new(settings.aoc.session_cookie.into_boxed_str());

    // Process each event as they come in.
    while let Some(event) = events_rx.recv().await {
        if matches!(event, Event::Shutdown) {
            break;
        }

        let fut = handle_event(
            event,
            http.clone(),
            Arc::clone(&board_id),
            Arc::clone(&session_cookie),
        );

        tokio::spawn(async {
            if let Err(e) = fut.await {
                eprintln!("failed handling event: {}", e);
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
) -> Result<cached::Return<LeaderboardStats>> {
    Ok(cached::Return::new(
        aoc::get_private_leaderboard_stats(session_cookie, 2020, leaderboard_id).await?,
    ))
}

async fn handle_event(
    event: Event,
    http: HttpClient,
    board_id: Arc<Box<str>>,
    session_cookie: Arc<Box<str>>,
) -> Result<()> {
    match event {
        Event::Ping(msg) => {
            info!("Ping message");
            http.create_message(msg.channel_id.into())
                .content(":ping_pong: Pong!")?
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

            let data = get_aoc_data(&session_cookie, &board_id).await?;

            debug!(
                "Retrieved data (cached: {}) -> constructing message",
                data.was_cached
            );
            let mut embed = EmbedBuilder::new()
                .title(format!("AoC Leaderboard [{}]", board_id))?
                .description(format!(
                    "Here is your current Leaderboard - Cached [{}]",
                    data.was_cached
                ))?;

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
                    )?
                    .inline()
                    .build(),
                );
            }
            debug!("sending discord message to {}", msg.channel_id);
            http.create_message(msg.channel_id.into())
                .embed(embed.build()?)?
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
