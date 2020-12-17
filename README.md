# AOC Bot

This is a simple bot you can spin up and watch your stats for the aoc competition. It uses the JSON API for private leaderboards.

## Description

This bot is supposed to be fast, low-overhead (saves money on server hosting) and easy to use. It has one purpose, to help show stats for the AoC challenges in your discord.

## Configuration

For configuration details, please check out the [CONFIGURATION](CONFIGURATION.md) document.

## What still needs to be implemented what can be improved?

- [ ] a cronjob like update mode
- [ ] a top 3 staircase
- [ ] stats for single participant
- [ ] better visualization
- [ ] docs so setup is faster

- [ ] Cooler name + mascot

You think you have a good idea? Well feel free to suggest it, or even do it yourself and create a pull request.

## Technologies used

- Rustlang
- tokio-rs
- twilight
- reqwest
- dotenv
- serde
- cached
- anyhow
