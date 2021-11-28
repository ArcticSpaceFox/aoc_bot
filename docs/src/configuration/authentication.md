# Authentication

Authentication is needed for the Advent of Code API to get leaderboard statistics as well as the Discord API to send messages as a bot. The configuration is located in `config/auth.toml` and this
bot will look for it in the current working directory from where it was executed.

## `aoc` - Advent of Code related settings

This section contains all information required to fetch leaderboard statistics from the Advent of
Code website.

### `board_id`

This is the board ID from the private leaderboard the bot should fetch information from. This can
be found by opening a leaderboard from the [AoC page]. Once opened the ID is the digits at the end
of the URL.

For example, if we would want the ID from the following leaderboard URL, the ID would be `12345`:

```txt
https://adventofcode.com/2020/leaderboard/private/view/12345
                                                       └─┬─┘
                                                   leaderboard ID
```

[AoC page]: https://adventofcode.com/2020/leaderboard/private

### `session_cookie`

The session cookie is the login cookie of a user that the AoC website uses to for its authentication
on all pages. To extract it, log in on the website and open the **storage inspector** of your
browser, locate the cookies area and look for an entry named `session`.

For example in Firefox this can be done by first opening _Hamburger menu -> Web Developer ->
Storage Inspector_. Then expand the cookies entry, click on the single entry within it and
double click on the cell in for the `session` entry labelled **Value** and lastly to copy it
(_CTRL+C_).

### `event_year`

This setting is to use the wanted AoC year, the API currently accepts the values `2015`, `2016`,
`2017`, `2018`, `2019`, `2020` and `2021`. That is, every year from the first AoC event until today.

## `discord` - Discord related settings

This section contains all authentication details needed to send send messages as a bot in Discord
through the service's API.

### `bot_token`

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

[Discord Developer Portal]: https://discord.com/developers/applications

### `schedule`

The bot allows to set a recurring automated leaderboard message without request from a user first.
To enable it crate a sub-section `[discord.schedule]` in the `config/auth.toml` file and set both of
the following fields.

#### `interval`

The interval describes how often the automated leaderboard should be sent, defined as a cron-job
string. For example to send the leaderboard every 2 hours you can use `0 0 */2 * * * *` as value.

This format basically defines the following time slots for each value, separated by spaces:

```txt
0 0 */2 * * * *
│ │  │  │ │ │ │
│ │  │  │ │ │ └─ year
│ │  │  │ │ └─ day of week
│ │  │  │ └─ month
│ │  │  └─ day of month
│ │  └─ hours
│ └─ minutes
└─ seconds
```

For further help and examples have a look at [crontab guru], although the website has a simpler
version without the _seconds_ and _year_ fields.

[crontab guru]: https://crontab.guru/

#### `channel_id`

The channel ID is the Discord channel where the leaderboard should be posted. In contrast to a user
sent command, where the reply is sent in the same channel as the user's command, the channel is
fixed as there is no way of determining the channel dynamically.

This ID can be found in Discord opening the context menu of a channel (right click) and selecting
**Copy ID** to copy it into the clipboard.

To see this option the developer mode must be enabled in the settings under
**App Settings > Advanced > Developer Mode**.

## Examples

Below are some example configuration for reference. **Please note** that you still must replace the
authentication related fields with real values to make the setup work.

### Basic

This is a sample configuration without any optional settings.

```toml
[aoc]
board_id = "12345"
session_cookie = "001122aabbcc"
event_year = 2021

[discord]
bot_token = "abcdef"
```

### Full

The following is a complete configuration file with all optional settings enabled.

```toml
[aoc]
board_id = "12345"
session_cookie = "001122aabbcc"
event_year = 2021

[discord]
bot_token = "abcdef"

[discord.schedule]
#          sec  min   hour   day of month   month   day of week   year
#          0    0     0      *             *       *             *
interval = "0 0 0 * * * *"
channel_id = 100
```
