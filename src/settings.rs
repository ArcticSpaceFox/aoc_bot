//! Authentication and logging settings for the bot.

use std::env;
use std::io::ErrorKind;
use std::num::NonZeroU64;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tokio::fs;
use tracing::Level;

/// Main structure that holds all the settings of this bot.
#[derive(Deserialize)]
pub struct Settings {
    /// Logger specific configuration.
    pub logging: Logging,
    /// Settings for the Advend of Code client.
    pub aoc: AdventOfCode,
    /// Discord related settings.
    pub discord: Discord,
}

/// All configuration for the logging of the bot, including different logging backends like a file
/// or the terminal.
#[derive(Deserialize)]
pub struct Logging {
    /// Logging settings for the terminal backend.
    pub terminal: Option<BaseLogger>,
    /// File backend settings.
    pub file: Option<FileLogger>,
}

impl Default for Logging {
    fn default() -> Self {
        Self {
            terminal: Some(BaseLogger {
                filter: Level::INFO,
            }),
            file: None,
        }
    }
}

/// The base logger describes the very basic settings that apply to each logging backend.
#[derive(Deserialize)]
pub struct BaseLogger {
    /// Maximum logging level that the backend outputs.
    #[serde(with = "serde_level")]
    pub filter: Level,
}

/// Logging configuration specific to file backends.
#[derive(Deserialize)]
pub struct FileLogger {
    /// base logging backend configuration.
    #[serde(flatten)]
    pub base: BaseLogger,
    /// Location of the file to write logs to.
    pub path: PathBuf,
}

/// All settings regarding the Advent of Code API.
#[derive(Deserialize)]
pub struct AdventOfCode {
    /// The leaderboard that is queried for current rankings and statistics.
    pub board_id: String,
    /// A session cookie to authenticate against the API. This is usually manually extracted with
    /// browser dev tools after logging into the website.
    pub session_cookie: String,
    /// The current event that is being tracked.
    pub event_year: u16,
}

/// Configuration for the Discord API.
#[derive(Deserialize)]
pub struct Discord {
    /// A token to authenticate against the Discord API as a bot and send messages.
    pub bot_token: String,
    #[serde(default)]
    pub schedule: Option<Schedule>,
}

#[derive(Deserialize)]
pub struct Schedule {
    pub interval: String,
    pub channel_id: NonZeroU64,
}

/// An intermediate structure for the authentication related settings that allows to parse them
/// separately and merge them into a single [Settings] structure later.
#[derive(Deserialize)]
struct Auth {
    aoc: AdventOfCode,
    discord: Discord,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            aoc: AdventOfCode {
                board_id: String::new(),
                session_cookie: String::new(),
                event_year: 2021,
            },
            discord: Discord {
                bot_token: String::new(),
                schedule: None,
            },
        }
    }
}

impl Settings {
    /// Create a new instance of the settings and fill it with the configuration from the
    /// `config/log.toml` and `config/auth.toml` files. All auth related settings are overwritten
    /// by env vars if they exist.
    pub async fn new() -> Result<Self> {
        let mut logging = load_toml::<Logging>("config/log.toml").await?;
        let Auth {
            mut aoc,
            mut discord,
        } = load_toml("config/auth.toml").await?;

        load_logging_envs(&mut logging)?;
        load_aoc_envs(&mut aoc)?;
        load_discord_envs(&mut discord)?;

        Ok(Self {
            logging,
            aoc,
            discord,
        })
    }
}

/// Overwrite logging settings with any provided env vars.
fn load_logging_envs(logging: &mut Logging) -> Result<()> {
    if let Ok(filter) = env::var("LOG_TERMINAL_FILTER") {
        let filter = filter
            .parse()
            .context("Failed to parse terminal logging filter")?;

        logging.terminal = Some(BaseLogger { filter });
    }

    if let (Ok(filter), Ok(path)) = (env::var("LOG_FILE_FILTER"), env::var("LOG_FILE_PATH")) {
        let filter = filter
            .parse()
            .context("Failed to parse file logging filter")?;
        let path = PathBuf::from(path);

        logging.file = Some(FileLogger {
            base: BaseLogger { filter },
            path,
        });
    }

    Ok(())
}

/// Overwrite Advent of Code settings with any provided env vars.
fn load_aoc_envs(aoc: &mut AdventOfCode) -> Result<()> {
    if let Ok(board_id) = env::var("AOC_BOARD_ID") {
        aoc.board_id = board_id;
    }

    if let Ok(session_cookie) = env::var("AOC_SESSION_COOKIE") {
        aoc.session_cookie = session_cookie;
    }

    if let Ok(event_year) = env::var("AOC_EVENT_YEAR") {
        aoc.event_year = event_year
            .parse::<u16>()
            .context("Failed to parse AOC event year")?;
    }

    Ok(())
}

/// Overwrite Discord settings with any provided env vars.
fn load_discord_envs(discord: &mut Discord) -> Result<()> {
    if let Ok(bot_token) = env::var("DISCORD_BOT_TOKEN") {
        discord.bot_token = bot_token;
    }

    if let (Ok(interval), Ok(channel_id)) = (
        env::var("DISCORD_SCHEDULE_INTERVAL"),
        env::var("DISCORD_SCHEDULE_CHANNEL_ID"),
    ) {
        let interval = interval
            .parse()
            .context("Failed to parse Discord schedule interval")?;
        let channel_id = channel_id
            .parse()
            .context("Failed to parse Discord schedule channel ID")?;

        discord.schedule = Some(Schedule {
            interval,
            channel_id,
        });
    }

    Ok(())
}

/// Load any deserializable structure from the given file path as TOML and provide helpful error
/// messages in case something goes wrong during the process.
async fn load_toml<T>(path: &str) -> Result<T>
where
    T: Default + DeserializeOwned,
{
    let content = match fs::read(path).await {
        Ok(content) => content,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(T::default()),
        Err(e) => {
            return Err(anyhow!(e)).context(format!("failed loading config file at '{}'", path))
        }
    };

    toml::from_slice(&content)
        .with_context(|| format!("failed to parse TOML config from '{}'", path))
}

/// A deserializer for the [LevelFilter] that allows to use it in [serde], as it doesn't provide
/// support for it out of the box.
mod serde_level {
    use std::fmt;

    use serde::de::{Deserializer, Visitor};
    use tracing::Level;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Level, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(LevelVisitor)
    }

    struct LevelVisitor;

    impl<'de> Visitor<'de> for LevelVisitor {
        type Value = Level;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("tracing logging level")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(match v {
                "error" => Level::ERROR,
                "warn" => Level::WARN,
                "info" => Level::INFO,
                "debug" => Level::DEBUG,
                "trace" => Level::TRACE,
                _ => return Err(E::custom(format!("unknown logging level `{v}`"))),
            })
        }
    }
}
