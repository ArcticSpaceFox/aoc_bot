use std::env;
use config::{ConfigError, Config, File, Environment};

#[derive(Debug)]
pub struct LoggersConfig {
    terminal: LoggerConfig,
    file: LoggerConfig,
}

#[derive(Debug, Deserialize)]
pub struct LoggerConfig {
    enabled: bool,
    filter: simplelog::LevelFilter,
}

#[derive(Debug, Deserialize)]
pub struct AoCConfig {
    board_id: String,
    session_cookie: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordConfig {
    bot_token: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    debug: bool,
    logger: LoggersConfig,
    aoc: AoCConfig,
    discord: DiscordConfig,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("config/default"))?;

        // Add in the current environment file
        // Default to 'development' env
        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        s.merge(File::with_name("config/local").required(false))?;

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("AOC"))?;
        s.merge(Environment::with_prefix("DISCORD"))?;

        // You may also programmatically change settings
        if s.get_bool("debug").unwrap_or(false) {
            s.set("logger.terminal.filter", "debug")?;
            s.set("logger.file.filter", "Trace")?;
        }

        // Now that we're done, let's access our configuration
        info!("debug: {:?}", s.get_bool("debug"));

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}