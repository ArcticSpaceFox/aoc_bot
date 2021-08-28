use anyhow::Result;
use futures_util::stream::StreamExt;
use log::{debug, error, info};
use tokio::sync::mpsc::Sender;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, Events, ShardScheme},
    Event, EventTypeFlags,
};
use twilight_http::Client as HttpClient;
use twilight_model::user::User;
use twilight_model::{channel::Message, gateway::Intents};

use crate::settings::Discord;

pub async fn start(settings: &Discord, sender: Sender<crate::models::Event>) -> Result<()> {
    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    let scheme = ShardScheme::Auto;
    debug!("Using scheme : {:?}", scheme);

    // Use intents to only receive guild message events.
    let (cluster, events) = Cluster::builder(settings.bot_token.clone(), Intents::GUILD_MESSAGES)
        .shard_scheme(scheme)
        .event_types(
            EventTypeFlags::MESSAGE_CREATE
                | EventTypeFlags::MESSAGE_DELETE
                | EventTypeFlags::MESSAGE_DELETE_BULK
                | EventTypeFlags::MESSAGE_UPDATE,
        )
        .build()
        .await?;

    debug!("Cluster set up");

    // Start up the cluster.
    let cluster_spawn = cluster;

    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        debug!("Spawning cluster");
        cluster_spawn.up().await;

        if let Err(e) = tokio::signal::ctrl_c().await {
            error!("Failed setting up CTRL+C listener: {}", e);
        }

        debug!("Stopping cluster");
        cluster_spawn.down();
    });

    // Since we only care about new messages, make the cache only
    // cache new messages.
    debug!("Setting up cache for twilight");
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    // Handle Discord events on a separate task.
    tokio::spawn(handle_events(events, cache, sender));

    Ok(())
}

async fn handle_events(
    mut events: Events,
    cache: InMemoryCache,
    sender: Sender<crate::models::Event>,
) {
    while let Some((shard_id, event)) = events.next().await {
        debug!("{} | Received event : {:?}", shard_id, event);
        cache.update(&event);

        match event {
            Event::MessageCreate(msg) => {
                let msg = match msg.content.as_str() {
                    "!ping" => crate::models::Event::Ping((shard_id, msg.0).into()),
                    "!aoc" => crate::models::Event::AdventOfCode((shard_id, msg.0).into()),
                    "!42" => crate::models::Event::FourtyTwo((shard_id, msg.0).into()),
                    _ => continue,
                };

                if sender.send(msg).await.is_err() {
                    return;
                }
            }
            Event::ShardConnected(conn) => info!("Connected on shard {}", conn.shard_id),
            _ => {}
        }
    }

    sender.send(crate::models::Event::Shutdown).await.ok();
}

pub fn new_client(settings: &Discord) -> HttpClient {
    HttpClient::new(settings.bot_token.clone())
}

impl From<(u64, Message)> for crate::models::Message {
    fn from((shard_id, m): (u64, Message)) -> Self {
        Self {
            shard_id,
            channel_id: m.channel_id.0,
            author: Some(m.author.into()),
        }
    }
}

impl From<User> for crate::models::Author {
    fn from(u: User) -> Self {
        Self {
            id: u.id.0,
            name: u.name,
        }
    }
}
