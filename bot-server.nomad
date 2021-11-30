job "bot-server" {
    datacenters = ["dc1"]

    # The meta key is used to associate arbitrary metadata with the job.
    meta {
        # This will allow for the job to actually redeploy if the
        # "bot-server" service changes. This is useful for rolling
        # updates.
        uuid = uuidv4()
    }

    # Specify this job to have rolling updates, two-at-a-time, with
    # 30 second intervals.
    update {
        max_parallel        = 2
        health_check        = "task_states"
        min_healthy_time    = "30s"
        healthy_deadline    = "5m"
        progress_deadline   = "10m"
        canary              = 1
        stagger             = "1m"
    }
    # All tasks for bot to run on the server
    group "bot" {
        # How many bots should be spinned up (in our case the aoc bot is just made for one)
        count = 1

        # The task for the actualy image
        task "server" {
            driver = "docker"

            config {
                image = "ghcr.io/arcticspacefox/aoc_bot:latest"
            }

            env {
                # AOC env vars
                AOC_BOARD_ID                = ""
                AOC_SESSION_COOKIE          = ""
                AOC_EVENT_YEAR              = "2021"
                # Discord env vars
                DISCORD_BOT_TOKEN           = ""
                # According to cron Syntax     sec  min   hour   day of month   month   day of week   year
                DISCORD_SCHEDULE_INTERVAL   = "0    0   4,10,16,22  *   *   *   *"
                DISCORD_SCHEDULE_CHANNEL_ID = ""
            }
        }
    }
}
