CREATE TYPE user_role AS ENUM  ('system_administrator', 'administrator');

-- Users that interact with the bot.
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY NOT NULL,
    service_username TEXT NOT NULL UNIQUE, -- The user's unique ID on the aforementioned service.
    user_role user_role NOT NULL -- The user's role with the bot.
);

CREATE TYPE arr_type AS ENUM ('sonarr', 'radarr');

-- Definitions of webhooks from an *arr (e.g. Sonarr or Radarr).
CREATE TABLE IF NOT EXISTS webhooks (
    id UUID PRIMARY KEY NOT NULL,
    arr_type arr_type NOT NULL, -- The *arr service this webhook is for (e.g. Sonarr or Radarr).
    username TEXT NOT NULL, -- The username that secures the webhook.
    password BYTEA NOT NULL, -- The password hash that secures the webhook.
    user_id UUID NOT NULL, -- The user that created and thus owns this webhook.
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Matrix rooms to post messages from webhooks in.
CREATE TABLE IF NOT EXISTS matrix_rooms (
    id UUID PRIMARY KEY NOT NULL,
    room_id TEXT NOT NULL, -- The room ID of the Matrix room (_not_ an alias).
    webhook_id UUID NOT NULL, -- The webhook to get message data from for messages posted in this room.
    FOREIGN KEY(webhook_id) REFERENCES webhooks(id) ON DELETE CASCADE
);