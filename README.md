# Yarrbot

A simple Matrix bot that listens for webhook notifications from [Sonarr](https://github.com/Sonarr/Sonarr) or [Radarr](https://github.com/Radarr/Radarr) 
and relays those notifications to one or more Matrix rooms.

This is a simple project being used by the author to start learning the Rust programming language. Yarrbot is beta 
software and comes with all of the risks associated with using beta software. Please read through the [LICENSE.txt](LICENSE.txt) 
before using.

The container examples in this README use [Podman](https://podman.io/) instead of Docker; if one would rather use Docker,
then for the most part simply substituting `podman` for `docker` in the command should be sufficient.

## Features

* Web API endpoint compatible with Sonarr/Radarr webhook requests; supports either `POST` or `PUT` requests.
* Configurable through a direct chat with the bot.
* Notifications arrive in both rich text (HTML) and plain text (formatted in Markdown).
* Designed to support the webhook notifications sent when Sonarr/Radarr:
  * Sees that a show/movie is available for download;
  * Has downloaded and imported some show/movie;
  * Has upgraded an existing show/movie to a better quality;
  * Has renamed a show/movie;
  * Deletes a series, movie, or episode file;
  * Detects when a health check has failed.

### Example Webhook Notification

![Example of a Notification from Yarrbot](docs/images/example_notification.png)

The above image is taken from the [Element desktop application](https://element.io/get-started). The information shown 
in the notification is designed with parity to the built-in Sonarr/Radarr email notifications.

## Setup

Yarrbot runs on Linux and requires a PostgreSQL database to store some of its configuration information. PostgreSQL 13
or greater is supported. There is no support for SQLite in Yarrbot at this time.

Yarrbot was designed to run on the same system as the Sonarr/Radarr instances. However, if this is not possible, then 
one _must_ put Yarrbot's web API behind a reverse proxy (e.g. Nginx) with a valid SSL certificate in place. Sonarr and 
Radarr authenticate with Yarrbot's web API via basic authentication and any failure to secure the API with HTTPS will 
result in those credentials being submitted to Yarrbot in plain text. 

In short, this means that without HTTPS, a bad actor can steal the webhook credentials and then submit data to Yarrbot 
to spam you with useless notifications.

### Run

#### Quick Start

Starting Yarrbot using a container image, binding to port `8081` on the host and mounting a managed volume to `/data`
in the container:

```
podman run -d \
    --name yarrbot \
    --secret yarrbot-db-url \
    --secret yarrbot-matrix-pass \
    -p 8081:8080 \
    -v yarrbot-storage:/data \
    -e YARRBOT_DATABASE_URL_FILE=/run/secrets/yarrbot-db-url \
    -e YARRBOT_LOG_FILTER=info \
    -e YARRBOT_MATRIX_USERNAME=yarrbot \
    -e YARRBOT_MATRIX_PASSWORD_FILE=/run/secrets/yarrbot-matrix-pass \
    -e YARRBOT_MATRIX_HOMESERVER_URL=https://matrix.example.org \
    -e YARRBOT_INITIALIZATION_USER=@you:example.org \
    --restart=always \
    ghcr.io/joeldewey/yarrbot:latest
```

#### Bare Metal

It is recommended to use systemd or another init system to manage the Yarrbot process. Configure the environment 
variable values (see the next section) for the user that Yarrbot will run under.

Please install the following dependencies before running Yarrbot for the first time:

* The Postgresql library (`libpq`)
* OpenSSL

No automated builds for generating an artifact of Yarrbot for bare metal installations are set up at this time. Therefore,
one will have to build Yarrbot themselves (see the [Build](README.md#Build) section) for this. 

#### Environment Variables

One may run Yarrbot by configuring a set of environment variables and then running the binary. Yarrbot does not require 
root permissions and should be run under a non-root user.

Any environment variable's name can be concatenated with `_FILE` to instruct Yarrbot to load the value from a file on 
the file system located at the path defined by the environment variable. See the "Quick Start" above for an example.

**Ensure that you secure the file appropriately (e.g. with the appropriate permissions).**

##### Required Environment Variables

* `YARRBOT_DATABASE_URL`: The connection string to the Postgresql database. Example: `postgres://username:password@localhost`
* `YARRBOT_MATRIX_HOMESERVER_URL`: The URL to the Matrix homeserver Yarrbot should connect to. Example: `https://matrix.example.org`
* `YARRBOT_MATRIX_USERNAME`: The username Yarrbot should use to connect to the Matrix homeserver.
* `YARRBOT_MATRIX_PASSWORD`: The password Yarrbot should use to connect to the Matrix homeserver.
* `YARRBOT_INITIALIZATION_USER`: Only required for the initial run of Yarrbot, this is the fully qualified Matrix User 
   ID for the user that will be configuring Yarrbot. Example: `@you:example.org`

##### Optional Environment Variables

* `YARRBOT_DATABASE_POOL_SIZE`: Yarrbot opens up a set of connections to the database and reuses them rather than 
   opening a new connection each time it needs one. This defaults to `20` connections.
* `YARRBOT_STORAGE_DIR`: An absolute path to some location on the file system for Yarrbot to store some of its runtime 
   configuration. Defaults to the directory the Yarrbot binary is in. If using the container image, this is exposed as 
   a volume mounted at `/data` within the container and should not be changed in favor of using your containers runtime's 
   native mounting functionality.
* `YARRBOT_WEB_PORT`: Some port for Yarrbot to bind the web API to when starting up. This defaults to `8080` if not set;
   if using the container image, this port is exposed and should be configured via your container runtime.
* `YARRBOT_LOG_FILTER`: Adjust the logging level of Yarrbot and its inner dependencies (crates); defaults to 
  `warn,yarrbot=info` which results in all messages from Yarrbot itself with an "informational" level or higher being 
  logged, but only "warning" or higher messages from Yarrbot's dependencies being logged. The default is recommended for
  most users. While Yarrbot uses the [`tracing` crate](https://tracing-rs.netlify.app/tracing/) for log functionality,
  the `tracing` crate uses the [`env_logger` crate's log level controls](https://docs.rs/env_logger/0.9.0/env_logger/#enabling-logging).

### Use

After Yarrbot starts up successfully for the first time, it will be configured to respond to direct messages from the  
user specified in the `YARRBOT_INITIALIZATION_USER` environment variable. Yarrbot responds to the following commands:

* `!yarrbot ping`: Testing command to which Yarrbot will reply with `pong`.
* `!yarrbot help`: Prints help information on the commands that Yarrbot supports, similar to this list.
* `!yarrbot sourcecode`: Get a link to the source code of Yarrbot.
* `!yarrbot webhook add roomOrAliasId username [password]`: Configures the bot to listen for webhook notifications to 
  relay to the given room. Requires that one specifies a username; a password may be supplied, otherwise Yarrbot will 
  generate one. This command will return an ID to use in the path to the webhook API supplied by the bot (e.g. `/api/v1/webhook/abcd1234`).
* `!yarrbot webhook list`: List the webhooks in the system.
* `!yarrbot webhook remove webhookId`: Removes a webhook by its ID, provided by the `webhook list` or `webhook add` 
  commands.

To set up either Sonarr or Radarr with Yarrbot:

1. Invite Yarrbot to some Matrix room to post messages to. It will automatically join the room upon an invitation from 
   the initial user.
2. Retrieve the Room ID or a Room Alias that Yarrbot's homeserver can resolve for the room. To get the Room ID via 
   Element, go to the room's "Room Settings" page, then in the "Advanced" page there will be an "Internal Room ID". 
   Copy the value starting with the exclamation point (`!`).
3. In a separate direct chat with Yarrbot, send it the following message: `!yarrbot webhook add !roomId:example.org webhook_user_1`
4. Yarrbot will reply with a confirmation and an ID, Username, and Password. Store these values in a secure location.
5. In Sonarr (or Radarr), go to "Settings" and then "Connect". Add a new "Webhook" connection.
6. Fill out the "Name", "Triggers", and "Tags" sections as appropriate. See [Sonarr's documentation](https://wiki.servarr.com/sonarr/settings#connection-triggers) for more details.
7. In the "URL" input, add the URL to Yarrbot's web API with the ID from Step (4) at the end of the route. For example, 
   a Yarrbot instance hosted on the same server as Sonarr and an ID of `abcd1234`: `http://localhost:8080/api/v1/webhook/abcd1234`
8. For "Method", select `POST`. For the "Username" and "Password", use the values from Step (4).
9. Click the "Test" button; one should see a message from Yarrbot in the Matrix room from Step (1). If not, double check
   your settings.
10. Click "Save"; Sonarr (or Radarr) will send one more test message and then save. From this point onward, whenever 
    Sonarr (or Radarr) perform one of the triggers with matching tags (if any) as defined in Step (6), it will send a 
    message to Yarrbot who will then relay that to your Matrix room of choice.

One may set up as many webhooks as they would like via this process.

# Build

If one would like to build Yarrbot themselves, then there are two options: building from the Dockerfile or building 
on bare metal.

A Dockerfile is provided to build a release version of Yarrbot as a container image:

```
podman build -t=yarrbot .
```

If one would rather build Yarrbot on bare metal, please make sure that your build machine has the following:

* The latest version of the Rust (older versions of Rust would probably work, but aren't tested)
* Postgresql development library (`libpq-dev`)
* OpenSSL
* `cmake`
* `make`
* `libstdc++`

It can then be built with the following command:

```
cargo build --release
```

There is no musl support currently.
