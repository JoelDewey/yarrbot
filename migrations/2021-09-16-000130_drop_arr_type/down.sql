-- arr_type is no longer needed as

CREATE TYPE arr_type AS ENUM ('sonarr', 'radarr');

ALTER TABLE IF EXISTS webhooks ADD COLUMN IF NOT EXISTS arr_type arr_type;