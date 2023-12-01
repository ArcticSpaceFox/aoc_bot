use anyhow::Result;
use log::{debug, error};
use tokio::sync::mpsc::Sender;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Config, Event, EventTypeFlags, Shard};
use twilight_http::Client as HttpClient;
use twilight_model::{channel::Message, gateway::Intents, user::User};
use twilight_model::gateway::{CloseFrame, ShardId};

use crate::settings::Discord;

pub async fn start(settings: &Discord, sender: Sender<crate::models::Event>) -> Result<()> {
    // Use intents to only receive guild message events.
    let config = Config::builder(settings.bot_token.clone(), Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT)
        .event_types(
            EventTypeFlags::MESSAGE_CREATE
                | EventTypeFlags::MESSAGE_DELETE
                | EventTypeFlags::MESSAGE_DELETE_BULK
                | EventTypeFlags::MESSAGE_UPDATE,
        )
        .build();
    let shard = Shard::with_config(ShardId::ONE, config);
    let shard_messages = shard.sender();

    debug!("Shard set up");

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            error!("Failed setting up CTRL+C listener: {}", e);
        }

        debug!("Stopping shard");
        if let Err(e) = shard_messages.close(CloseFrame::NORMAL) {
            error!("Failed closing shard: {}", e);
        }
    });

    // Since we only care about new messages, make the cache only
    // cache new messages.
    debug!("Setting up cache for twilight");
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    // Handle Discord events on a separate task.
    tokio::spawn(handle_events(shard, cache, sender));

    Ok(())
}

async fn handle_events(
    mut shard: Shard,
    cache: InMemoryCache,
    sender: Sender<crate::models::Event>,
) {
    while let Ok(event) = shard.next_event().await {
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
                    break;
                }
            }
            Event::GatewayClose(_) => {
                debug!("Shutting down");
                break;
            }
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
