# Environment variables

In addition (or as replacement) for configuration files, all possible settings can be done through
environment variables. For details about each option, please refer to the appropriate section of the
config files.

## Available values

The following sections list the available environment variables with links to the respective config
file settings.

### Advent of Code

These are the AoC related settings from the `auth.toml` file.

- `AOC_BOARD_ID`: [`aoc.board_id`](authentication.md#board_id)
- `AOC_SESSION_COOKIE`: [`aoc.session_cookie`](authentication.md#session_cookie)
- `AOC_EVENT_YEAR`: [`aoc.event_year`](authentication.md#event_year)

### Discord

These are the Discord related settings from the `auth.toml` file.

- `DISCORD_BOT_TOKEN`: [`discord.bot_token`](authentication.md#bot_token)
- `DISCORD_SCHEDULE_INTERVAL`: [`discord.schedule.interval`](authentication.md#interval)
- `DISCORD_SCHEDULE_CHANNEL_ID`: [`discord.schedule.channel_id`](authentication.md#channel_id)

**Please note**: `DISCORD_SCHEDULE_INTERVAL` and `DISCORD_SCHEDULE_CHANNEL_ID` must both be set
together. Setting only one won't have any effect.

### Logging

These are the logging related settings from the `log.toml` file.

- `LOG_TERMINAL_FILTER`: [`terminal.filter`](logging.md#terminal---terminal-output)
- `LOG_FILE_FILTER`: [`file.filter`](logging.md#file---file-output)
- `LOG_FILE_PATH`: [`file.path`](logging.md#path)

**Please note**: `LOG_FILE_FILTER` and `LOG_FILE_PATH` must both be set together. Setting only one
won't have any effect.

## Using an `.env` file

The application allows to set environment variables through a file call `.env`. This is a common
approach that allows to configure applications through env vars without having to define them every
time before execution.

Contents of the file are pairs of env var names and values, separated by an equal sign `=` and each
entry separated by newlines `\n`. Comments can be added by prefixing values with a `#`. For example:

```sh
# .env file contents
AOC_BOARD_ID=12345
AOC_SESSION_COOKIE=00112233aabbccdd
```

environment variables are automatically loaded from this file if it can be found in the current
working directory. Any already defined env var will not be overriden by this file.
