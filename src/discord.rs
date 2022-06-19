use anyhow::Result;
use futures_util::stream::StreamExt;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, info};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{shard::Events, Event, EventTypeFlags, Shard};
use twilight_http::Client as HttpClient;
use twilight_model::{channel::Message, gateway::Intents, user::User};

use crate::settings::Discord;

pub async fn start(settings: &Discord, sender: Sender<crate::models::Event>) -> Result<()> {
    // Use intents to only receive guild message events.
    let (shard, events) = Shard::builder(settings.bot_token.clone(), Intents::GUILD_MESSAGES)
        .event_types(
            EventTypeFlags::MESSAGE_CREATE
                | EventTypeFlags::MESSAGE_DELETE
                | EventTypeFlags::MESSAGE_DELETE_BULK
                | EventTypeFlags::MESSAGE_UPDATE,
        )
        .build()
        .await?;

    shard.start().await?;

    debug!("Shard set up");

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            error!("Failed setting up CTRL+C listener: {}", e);
        }

        debug!("Stopping shard");
        shard.shutdown();
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
    while let Some(event) = events.next().await {
        debug!("Received event : {:?}", event);
        cache.update(&event);

        match event {
            Event::MessageCreate(msg) => {
                let msg = match msg.content.as_str() {
                    "!ping" => crate::models::Event::Ping(msg.0.into()),
                    "!aoc" => crate::models::Event::AdventOfCode(msg.0.into()),
                    "!42" => crate::models::Event::FourtyTwo(msg.0.into()),
                    "!top3" => crate::models::Event::TopThree(msg.0.into()),
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

pub fn new_client(token: String) -> HttpClient {
    HttpClient::new(token)
}

impl From<Message> for crate::models::Message {
    fn from(m: Message) -> Self {
        Self {
            channel_id: m.channel_id.into(),
            author: Some(m.author.into()),
            timestamp: Some(m.timestamp),
        }
    }
}

impl From<User> for crate::models::Author {
    fn from(u: User) -> Self {
        Self {
            id: u.id.into(),
            name: u.name,
        }
    }
}
