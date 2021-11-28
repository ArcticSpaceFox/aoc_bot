# Docker

The easiest way to deploy this bot is through Docker. It allows to run it independent of the
software installed on the system as Docker isolates its _containers_ from the host. This project
comes with a `Dockerfile` so you only need to build it and are good to go.

## Building

Creating the image is straight forward and the same as for any typical Dockerfile. Just navigate to
the root of the project and run the following command to build the image:

```sh
docker build -t aoc_bot .
```

This will create the image, tagged with the name `aoc_bot`. You can choose any name you like.

## Running

The built image can be run either directly through Docker, Docker Compose or by other systems that
integrate with Docker (for example Terraform, Ansible, Kubernetes, ...).

For example ,the following command runs the bot, assuming all settings are configured through the
[authentication](configuration/authentication.md) and [logging](configuration/logging.md) files.

```sh
docker run --rm -it \
  -v $PWD/config/auth.toml:/data/config/auth.toml:ro \
  -v $PWD/config/log.toml:/data/config/log.toml:ro \
  aoc_bot
```

The following version is an alternative using an `.env` file to configure the bot as described in
the [environment variables](configuration/environment-variables.md##using-an-env-file) section.

```sh
docker run --rm -it --env-file .env aoc_bot
```

## Docker compose

The project comes with a basic Docker Compose configuration that can be used to run this bot through
Docker. It is an equivalent to the above commands but allows easier execution without having to
remember all necessary command line options.

```sh
docker-compose up --build
```

It will build the image if it isn't available in the local storage and then execute the bot.
Settings are loaded by mounting the config files as well as using the env var file to load
environment files for configuration.
