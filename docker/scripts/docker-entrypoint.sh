#!/usr/bin/env bash

set -e

PUID=${PUID:-1000}
PGID=${PGID:-1000}

groupmod -o -g "$PGID" yarrbot
usermod -o -u "$PUID" yarrbot

if [ "$1" = 'yarrbot' ]; then
    exec gosu yarrbot "$@"
fi

exec "$@"
