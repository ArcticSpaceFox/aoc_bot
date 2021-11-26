# Configuration

The configuration for this bot is split into two distinct files that are expected under the `config`
folder of the working directory. All available settings are explained in the following sections.

## Authentication

Authentication is needed for the Advent of Code API to get leaderboard statistics as well as the Discord API to send messages as a bot. The configuration is located in `config/auth.toml` and this
bot will look for it in the current working directory from where it was executed.

All settings are required to be set but can be overridden with environment variables. Therefore, it
is possible to keep the default configuration file with empty values for all settings and set all of
them through environment variables.

### `aoc` - Advent of Code

This section contains all information required to fetch leaderboard statistics from the Advent of
Code website.

#### `board_id`

This is the board ID from the private leaderboard the bot should fetch information from. This can
be found by opening a leaderboard from the [AoC page]. Once opened the ID is the digits at the end
of the URL.

For example, if we would want the ID from the following leaderboard URL, the ID would be `12345`:

```txt
https://adventofcode.com/2020/leaderboard/private/view/12345
                                                       └─┬─┘
                                                   leaderboard ID
```

This entry can be set with the `AOC_BOARD_ID` environment variable as alternative to the config
file.

[AoC page]: https://adventofcode.com/2020/leaderboard/private

#### `session_cookie`

The session cookie is the login cookie of a user that the AoC website uses to for its authentication
on all pages. To extract it, log in on the website and open the **storage inspector** of your
browser, locate the cookies area and look for an entry named `session`.

For example in Firefox this can be done by first opening _Hamburger menu -> Web Developer ->
Storage Inspector_. Then expand the cookies entry, click on the single entry within it and
double click on the cell in for the `session` entry labelled **Value** and lastly to copy it
(_CTRL+C_).

This entry can be set with the `AOC_SESSION_COOKIE` environment variable as alternative to the
config file.

#### `event`

This setting is to use the wanted aoc year, the API currently accepts the value `2021`. As of now 
previous years will return an error. Be aware of this! 

One way to configure it is by setting the corresponding environment variable `AOC_EVENT`, the value
should be number! Another way is in the `config/auth.toml` in the `[aoc]` section.

```yaml
[aoc]
board_id = "..."
session_cookie = "..."
event= 2021
```

### `discord` - Discord

This section contains all authentication details needed to send send messages as a bot in Discord
through the service's API.

#### `bot_token`

The only required setting is this bot token which allows to authenticate as a bot. It can be
retrieved from the Discord Developer Portal as follows:

- First navigate to the [Discord Developer Portal] and log in if you haven't yet.
- Create a new application or use an existing one and open it.
- In the navigation select the **Bot** area. Here you can find your **bot token** that you need for
  this setting.
- To enable the application for a Discord Server, navigate to the **OAuth2** section to generate a
  URL for the authorization process.
  - In **Scopes** select the `bot` scope.
  - In **Bot Permissions** select the `Text Messages` and `Read Message History` permission.
  - Copy the link from the **Scopes** section and open it in a tab.
  - Select the server where you want to install the application.

This entry can be set with the `DISCORD_BOT_TOKEN` environment variable as alternative to the config
file.

[Discord Developer Portal]: https://discord.com/developers/applications

#### `post_interval`

Cron-job-like setting to post the current stats every x amount of time automatically. To configure
go to the `config/auth.toml` and adjust the `discord.schedule.interval`. Needs to be a valid cron string.

## Logging

The bot logs several messages while it is running that can help to understand its current state and
aid in debugging errors. This logging section controls where this information is written to and at
what verbosity level.

In addition, logs can be written to different backend at the same time. Further details are written
below for each supported backend.

To disable any of the backends, comment out or remove the whole section of a backend.

### `filter`

The filter defines the verbosity of error messages. For a production setup it's recommended to set
it to `warn` for the terminal output and `info` for the file output. This default configuration can
be found at the project's [log.toml](config/log.toml).

This setting is available to all log backends and the following values are accepted, ranked from
least to most verbose:

- `error`: most silent version, only log critical errors.
- `warn`: log critical errors and warnings.
- `info`: log informative messages, warnings and errors.
- `debug`: write messages helpful for debugging purposes and all above type of messages.
- `trace`: most verbose, logs very detailed information plus all the other levels.

### `terminal` - Terminal output

This backend writes logs directly to the terminal where the bot is running. It has only one setting,
the `filter`.

### `file` - File output

This backend writes logs to a file instead of the terminal and has two settings, the `filter` and
the `file`.

#### `path`

This setting defines the path of a file that the backend will append logs to. If the file is missing
it will be automatically created.
