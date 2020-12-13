#[macro_use]
extern crate log;
extern crate simplelog;

use anyhow::Result;
use aoc_bot::YearEvent;
mod settings;

use cached::proc_macro::cached;
use reqwest::header;
use simplelog::*;
use std::fs::File;

use tokio::stream::StreamExt;
use twilight_cache_inmemory::{EventType, InMemoryCache};
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;

#[tokio::main]
async fn main() -> Result<()> {
    // Loading .env file
    dotenv::dotenv().ok();
    // Load settings file
    let settings = settings::Settings::new()?;

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

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        debug!("{} | Received event : {:?}", shard_id, event);
        // Update the cache with the event.
        cache.update(&event);

        let fut = handle_event(
            shard_id,
            event,
            http.clone(),
            settings.aoc.board_id.clone(),
            settings.aoc.session_cookie.clone(),
        );

        tokio::spawn(async {
            if let Err(e) = fut.await {
                eprintln!("failed handling event: {}", e);
            }
        });
    }

    Ok(())
}

#[cached(time = 7200, result = true, with_cached_flag = true)]
async fn get_aoc_data(
    request_url: String,
    session_cookie: String,
) -> Result<cached::Return<YearEvent>> {
    debug!("Attempting : {}", request_url);
    let cookie = cookie::Cookie::build("session", session_cookie).finish();
    let response = reqwest::Client::new()
        .get(&request_url)
        .header(header::COOKIE, cookie.to_string())
        .send()
        .await?;
    debug!("Retrieved DATA");

    // Read the response body as text into a string and print it.
    let data = response.json::<YearEvent>().await?;
    debug!("Parsed DATA");

    Ok(cached::Return::new(data))
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: HttpClient,
    lid: String,
    session_cookie: String,
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
                let request_url = format!(
                    "https://adventofcode.com/2020/leaderboard/private/view/{}.json",
                    lid
                );
                info!(
                    "Request from ({}) {} to get aoc board",
                    msg.author.id, msg.author.name
                );
                let data = get_aoc_data(request_url, session_cookie).await?;

                debug!(
                    "Retrieved data (cached: {}) -> constructing message",
                    data.was_cached
                );
                let mut embed = EmbedBuilder::new()
                    .title(format!("AoC Leaderboard [{}]", lid))?
                    .description(format!(
                        "Here is your current Leaderboard - Cached [{}]",
                        data.was_cached
                    ))?;

                let mut uvec: Vec<_> = data.members.iter().collect();
                uvec.sort_by(|a, b| b.1.cmp(a.1));

                for (idx, user) in uvec.iter().enumerate() {
                    embed = embed.field(
                        EmbedFieldBuilder::new(
                            format!(
                                "#{} - {} - {} score",
                                idx + 1,
                                user.1.name,
                                user.1.local_score
                            ),
                            format!("Solved {} Challenges", user.1.stars),
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
