//! Advent of Code API to retrieve leaderboard statistics.

use std::collections::HashMap;

use anyhow::Result;
use chrono::prelude::*;
use reqwest::header::{self, HeaderMap};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct LeaderboardStats {
    pub event: String,
    pub owner_id: String,
    pub members: HashMap<String, User>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    pub id: String,
    pub name: Option<String>,
    pub stars: u32,
    pub local_score: u32,
    pub global_score: u32,
    pub completion_day_level: CompletionDayLevel,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CompletionDayLevel {
    #[serde(rename = "1")]
    pub value: Option<Day>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Day {
    #[serde(rename = "1")]
    pub part1: Challenge,
    #[serde(rename = "2")]
    pub part2: Option<Challenge>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Challenge {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub get_star_ts: DateTime<Utc>,
}

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
}

impl Client {
    pub fn new(session_cookie: &str) -> Result<Self> {
        let cookie = format!("session={}", session_cookie);

        let mut headers = HeaderMap::with_capacity(1);
        headers.insert(header::COOKIE, cookie.try_into()?);

        Ok(Self {
            http: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
        })
    }

    /// Get the latest statistics from a private leaderboard. It is asked by the AoC website owners
    /// to not request this data more often than every 15 minutes.
    pub async fn get_private_leaderboard_stats(
        &self,
        event: u16,
        leaderboard_id: &str,
    ) -> Result<LeaderboardStats> {
        let url = format!(
            "https://adventofcode.com/{}/leaderboard/private/view/{}.json",
            event, leaderboard_id
        );

        self.http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(Into::into)
    }
}
