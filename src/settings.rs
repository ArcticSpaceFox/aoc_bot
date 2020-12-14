use std::{env, str::FromStr};

use config::{Config, ConfigError, Environment, File};
use log::info;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(remote = "simplelog::LevelFilter", rename_all = "lowercase")]
pub enum LevelFilterDef {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggerConfigs {
    pub terminal: LoggerConfig,
    pub file_path: String,
    pub file: LoggerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggerConfig {
    pub enabled: bool,
    // (Off, Error, Warn, Info, Debug, Trace)
    #[serde(with = "LevelFilterDef")]
    pub filter: simplelog::LevelFilter,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AoCConfig {
    pub board_id: String,
    pub session_cookie: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DiscordConfig {
    pub bot_token: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RunMode {
    Development,
    Production,
}

impl FromStr for RunMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            _ => Err(()),
        }
    }
}

impl ToString for RunMode {
    fn to_string(&self) -> String {
        match self {
            Self::Development => "development",
            Self::Production => "production",
        }
        .into()
    }
}

impl Default for RunMode {
    fn default() -> Self {
        Self::Development
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    #[serde(default)]
    pub run_mode: RunMode,
    pub logger: LoggerConfigs,
    pub aoc: AoCConfig,
    pub discord: DiscordConfig,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("config/default"))?;

        // Add in the current environment file
        // Default to 'development' env
        // Note that this file is _optional_
        let env = RunMode::from_str(&env::var("RUN_MODE").unwrap_or_else(|_| "development".into()))
            .map_err(|_| ConfigError::Message("invalid run mode".into()))?;

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        s.merge(File::with_name("config/local").required(false))?;

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("AOC"))?;
        s.merge(Environment::with_prefix("DISCORD"))?;

        // You may also programmatically change settings
        if env == RunMode::Development {
            s.set("logger.terminal.enabled", true)?;
            s.set("logger.terminal.filter", "info")?;
            s.set("logger.file.enabled", true)?;
            s.set("logger.file.filter", "debug")?;
        }
        s.merge(File::with_name(&format!("config/{}.toml", env.to_string())).required(false))?;

        // Now that we're done, let's access our configuration
        info!("Configured in mode: {}", env.to_string());

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}
