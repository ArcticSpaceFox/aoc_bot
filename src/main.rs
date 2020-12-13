use std::fs::File;
use std::iter::FromIterator;
use std::sync::Arc;

use anyhow::Result;
use cached::proc_macro::cached;
use log::{debug, info};
use simplelog::{CombinedLogger, Config, SharedLogger, TermLogger, TerminalMode, WriteLogger};
use tokio::stream::StreamExt;
use twilight_cache_inmemory::{EventType, InMemoryCache};
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;

use aoc_bot::aoc::{self, LeaderboardStats, User};
use aoc_bot::settings::Settings;

#[tokio::main]
async fn main() -> Result<()> {
    // Loading .env file
    dotenv::dotenv().ok();
    // Load settings file
    let settings = Settings::new()?;

    // Setting up an combined logger which will log to the terminal and a file
    let _logger = CombinedLogger::init({
        let mut buf = Vec::<Box<dyn SharedLogger>>::new();
        // TODO: Read log level from config
        match settings.logger.terminal.enabled {
            true => buf.push(TermLogger::new(
                settings.logger.terminal.filter,
                Config::default(),
                TerminalMode::Mixed,
            )),
            false => debug!("Terminal logger disabled"),
        };
        buf.push(WriteLogger::new(
            settings.logger.file.filter,
            Config::default(),
            File::create(settings.logger.file_path).unwrap(),
        ));
        buf
    })
    .expect("Logger failed to set up");

    info!("Starting ...");
    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    let scheme = ShardScheme::Auto;
    debug!("Using scheme : {:?}", scheme);

    // Use intents to only receive guild message events.
    let cluster = Cluster::builder(settings.discord.bot_token.clone(), Intents::GUILD_MESSAGES)
        .shard_scheme(scheme)
        .build()
        .await?;

    debug!("Cluster set up");

    // Start up the cluster.
    let cluster_spawn = cluster.clone();

    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        debug!("Spawning cluster");
        cluster_spawn.up().await;
    });

    // HTTP is separate from the gateway, so create a new client.
    debug!("Setting up http client for twilight");
    let http = HttpClient::new(settings.discord.bot_token.clone());

    // Since we only care about new messages, make the cache only
    // cache new messages.
    debug!("Setting up cache for twilight");
    let cache = InMemoryCache::builder()
        .event_types(
            EventType::MESSAGE_CREATE
                | EventType::MESSAGE_DELETE
                | EventType::MESSAGE_DELETE_BULK
                | EventType::MESSAGE_UPDATE,
        )
        .build();

    let mut events = cluster.events();

    let board_id = Arc::new(settings.aoc.board_id.into_boxed_str());
    let session_cookie = Arc::new(settings.aoc.session_cookie.into_boxed_str());

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        debug!("{} | Received event : {:?}", shard_id, event);
        // Update the cache with the event.
        cache.update(&event);

        let fut = handle_event(
            shard_id,
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
        aoc::get_private_leaderboard_stats(&session_cookie, 2020, &leaderboard_id).await?,
    ))
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: HttpClient,
    board_id: Arc<Box<str>>,
    session_cookie: Arc<Box<str>>,
) -> Result<()> {
    match event {
        Event::MessageCreate(msg) => match msg.content.as_str() {
            "!ping" => {
                info!("Ping message");
                http.create_message(msg.channel_id)
                    .content(":ping_pong: Pong!")?
                    .await?;
            }
            "!aoc" => {
                info!(
                    "Request from ({}) {} to get aoc board",
                    msg.author.id, msg.author.name
                );
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

                let mut uvec = Vec::from_iter(data.members.values());
                uvec.sort_by(|a, b| b.local_score.cmp(&a.local_score));

                for (idx, user) in uvec.iter().enumerate() {
                    embed = embed.field(
                        EmbedFieldBuilder::new(
                            format!("#{} - {} - {} score", idx + 1, user.name, user.local_score),
                            format!(
                                "⭐ Solved {} Challenges\n⏱️ Last at {}",
                                user.stars,
                                latest_challenge(&user)
                            ),
                        )?
                        .inline()
                        .build(),
                    );
                }
                debug!("sending discord message to {}", msg.channel_id);
                http.create_message(msg.channel_id)
                    .embed(embed.build()?)?
                    .await?;
            }
            "!42" => {
                info!("42 message");
                http.create_message(msg.channel_id)
                    .content(
                        ":exploding_head: \
                    The Answer to the Ultimate Question of Life, \
                    the Universe, and Everything is 42",
                    )?
                    .await?;
            }
            _ => {}
        },
        Event::ShardConnected(_) => {
            info!("Connected on shard {}", shard_id);
        }
        // Other events here...
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
        Some(ts) => ts.to_string(),
    }
}
