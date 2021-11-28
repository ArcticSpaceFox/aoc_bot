# AOC Bot

This is a simple bot you can spin up and watch your stats for the aoc competition. It uses the JSON
API for private leaderboards.

## Description

This bot is supposed to be fast, low-overhead (saves money on server hosting) and easy to use. It
has one purpose, to help show stats for the AoC challenges in your discord.

## Configuration & Docker

For further details, configuration and Docker setup, please have a look at the
[book](https://ArcticSpaceFox.github.io/aoc_bot).

## What still needs to be implemented what can be improved?

- [x] a top 3 staircase
- [ ] stats for single participant
- [ ] better visualization
- [x] docs so setup is faster

- [ ] Cooler name + mascot

You think you have a good idea? Well feel free to suggest it, or even do it yourself and create a
pull request.

## Technologies used

- Rustlang
- tokio-rs
- twilight
- reqwest
- dotenv
- serde
- cached
- anyhow

## License

This project is licensed under the [AGPL-3.0 License](LICENSE) (or
<https://www.gnu.org/licenses/agpl-3.0.html>).
