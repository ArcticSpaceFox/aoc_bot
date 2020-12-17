# AOC Bot

This is a simple bot you can spin up and watch your stats for the aoc competition. It uses the JSON API for private leaderboards.

## Description

This bot is supposed to be fast, low-overhead (saves money on server hosting) and easy to use. It has one purpose, to help show stats for the AoC challenges in your discord.

## Configuration

For configuration details, please check out the [CONFIGURATION](CONFIGURATION.md) document.

## Docker

The project provides a `Dockerfile` and a sample `docker-compose.yml`. You can build the and run it
with Docker directly:

```sh
docker build -t aoc_bot .
docker run --rm -it --env-file .env aoc_bot
```

Or you can run it with Docker Compose:

```sh
docker-compose up
```

All the above commands assume your auth configuration is set as env vars in a `.env` file.

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
