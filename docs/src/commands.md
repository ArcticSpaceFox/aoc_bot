# Commands

The main feature of this bot is the [`!aoc`](#aoc) command, but it supports some other commands as
well. Some of them are related to AoC and some are just for fun or testing purposes.

## `!ping`

The ping command allows to check how long the bot needs to interact with the Discord APIs. After
receiving the user's command the bot will respond with a simple message initially and then update
it immediately afterwards to measure the time it takes between sending 2 commands to Discord.

## `!aoc`

The main command, giving a large overview of the current members in the configured leaderboard. It
renders a list of all members, sorted by their star count together with several statistics.

## `!top3`

This command renders a top 3 stair case with the first 3 members that have the highest star count.
Currently the stair case is rendered as ASCII art.

## `!42`

A fun command, for people who read or watched the **The Hitchhiker's Guide to the Galaxy**
book/movie.
