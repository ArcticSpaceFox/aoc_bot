use std::env;
use std::error::Error;

use cached::proc_macro::cached;

use anyhow::{Context, Result};
use aoc_bot::YearEvent;
use reqwest::header;
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
    dotenv::dotenv().ok();

    println!("Configuring ...");
    let lid = env::var("AOC_BOARD_ID").context("AOC_BOARD_ID env var missing")?;
    let session_cookie =
        env::var("AOC_SESSION_COOKIE").context("AOC_SESSION_COOKIE env var missing")?;
    let token = env::var("DISCORD_BOT_TOKEN").context("DISCORD_BOT_TOKEN env var missing")?;

    println!("Starting ...");

    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    let scheme = ShardScheme::Auto;

    // Use intents to only receive guild message events.
    let cluster = Cluster::builder(token.clone(), Intents::GUILD_MESSAGES)
        .shard_scheme(scheme)
        .build()
        .await?;

    // Start up the cluster.
    let cluster_spawn = cluster.clone();

    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // HTTP is separate from the gateway, so create a new client.
    let http = HttpClient::new(token.clone());

    // Since we only care about new messages, make the cache only
    // cache new messages.
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
        // Update the cache with the event.
        cache.update(&event);

        tokio::spawn(handle_event(
            shard_id,
            event,
            http.clone(),
            lid.clone(),
            session_cookie.clone(),
        ));
    }

    Ok(())
}

#[cached(time = 7200, result = true, with_cached_flag = true)]
async fn get_aoc_data(
    request_url: String,
    session_cookie: String,
) -> Result<cached::Return<YearEvent>> {
    println!("Attempting : {}", request_url);
    let cookie = cookie::Cookie::build("session", session_cookie).finish();
    let response = reqwest::Client::new()
        .get(&request_url)
        .header(header::COOKIE, cookie.to_string())
        .send()
        .await?;
    println!("Retrieved DATA");

    // Read the response body as text into a string and print it.
    let data = response.json::<YearEvent>().await?;
    println!("Parsed DATA");

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
        Event::MessageCreate(msg) if msg.content == "!ping" => {
            http.create_message(msg.channel_id)
                .content("Pong!")?
                .await?;
        }
        Event::MessageCreate(msg) if msg.content == "!aoc" => {
            let request_url = format!(
                "https://adventofcode.com/2020/leaderboard/private/view/{}.json",
                lid
            );
            let data = get_aoc_data(request_url, session_cookie).await?;
            println!("From Cache : {}", data.was_cached);
            println!("Creating embed");
            let mut embed = EmbedBuilder::new()
                .title(format!("AoC Leaderboard [{}]", lid))?
                .description(format!(
                    "Here is your current Leaderboard - Cached [{}]",
                    data.was_cached
                ))?;

            let mut uvec: Vec<_> = data.members.iter().collect();
            uvec.sort_by(|a, b| b.1.cmp(a.1));

            for (idx, user) in uvec.iter().enumerate() {
                println!("#{} - {} - {} stars", idx + 1, user.1.name, user.1.stars);
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
            println!("sending message");
            http.create_message(msg.channel_id)
                .embed(embed.build()?)?
                .await?;
        }
        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
        }
        // Other events here...
        _ => {}
    }

    Ok(())
}
