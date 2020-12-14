use std::collections::HashMap;

use anyhow::Result;
use cookie::Cookie;
use reqwest::header;
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
    pub name: String,
    pub stars: u32,
    pub local_score: u32,
    pub global_score: u32,
    pub completion_day_level: HashMap<String, Day>,
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
    pub get_star_ts: String,
}

pub async fn get_private_leaderboard_stats(
    session_cookie: &str,
    event: u16,
    leaderboard_id: &str,
) -> Result<LeaderboardStats> {
    let url = format!(
        "https://adventofcode.com/{}/leaderboard/private/view/{}.json",
        event, leaderboard_id
    );
    let cookie = Cookie::build("session", session_cookie)
        .finish()
        .to_string();

    let response = reqwest::Client::new()
        .get(&url)
        .header(header::COOKIE, cookie)
        .send()
        .await?;

    Ok(response.json().await?)
}
