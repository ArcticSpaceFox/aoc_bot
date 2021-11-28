# Logging

The bot logs several messages while it is running that can help to understand its current state and
aid in debugging errors. This logging section controls where this information is written to and at
what verbosity level.

In addition, logs can be written to different backend at the same time. Further details are written
below for each supported backend.

To disable any of the backends, comment out or remove the whole section of a backend.

## Filter levels

The filter defines the verbosity of error messages. For a production setup it's recommended to set
it to `warn` for the terminal output and `info` for the file output. This default configuration can
be found at the project's `config/log.toml`.

This setting is available to all log backends and the following values are accepted, ranked from
least to most verbose:

- `error`: most silent version, only log critical errors.
- `warn`: log critical errors and warnings.
- `info`: log informative messages, warnings and errors.
- `debug`: write messages helpful for debugging purposes and all above type of messages.
- `trace`: most verbose, logs very detailed information plus all the other levels.

## `terminal` - Terminal output

This backend writes logs directly to the terminal where the bot is running. It has only one setting,
the `filter`. Refer to the [Filter levels](#filter-levels) area for possible values.

## `file` - File output

This backend writes logs to a file instead of the terminal and has two settings, the `filter` and
the `path`. Refer to the [Filter levels](#filter-levels) area for possible values of the `filter`
field.

### `path`

This setting defines the path of a file that the backend will append logs to. If the file is missing
it will be automatically created.

## Examples

### Terminal only

Only terminal output is enabled, with debug or worse logs, and no log file will be created.

```toml
[terminal]
filter = "debug"
```

### File only

The terminal output will be completely silent and debug or worse logs will be logged to the file
`aocbot.log`.

```toml
[file]
filter = "debug"
path = "aocbot.log"
```

### Both

A combination of the above examples, logging only warnings and errors to the terminal and more detailed logs to the `aocbot.log` file.

```toml
[terminal]
filter = "warn"

[file]
filter = "debug"
path = "aocbot.log"
```
