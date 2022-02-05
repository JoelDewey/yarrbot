FROM rust:1.58 as builder

WORKDIR /usr/src/myapp

COPY ./ ./

# libstdc++ is already installed in the base image.
RUN apt-get update && apt-get install -y cmake make libpq-dev
RUN cargo build --release
RUN strip ./target/release/yarrbot

FROM debian:buster-slim

RUN addgroup --system --gid 1000 yarrbot && \
    adduser --system --no-create-home --shell /bin/false --uid 1000 --gid 1000 yarrbot && \
    apt-get update && \
    apt-get install -y ca-certificates libpq5 gosu && \
    mkdir /app /data && \
    chown -R yarrbot:yarrbot /app && \
    chown yarrbot:yarrbot /data && \
    rm -rf /var/lib/apt/lists/* && \
    update-ca-certificates
COPY --chown=yarrbot:yarrbot ./docker/scripts/docker-entrypoint.sh /usr/bin/docker-entrypoint.sh
COPY --chown=yarrbot:yarrbot --from=builder /usr/src/myapp/target/release/yarrbot /usr/bin/yarrbot

VOLUME ["/data"]

ENV YARRBOT_STORAGE_DIR=/data
ENV YARRBOT_WEB_PORT=8080
ENV PUID=1000
ENV PGID=1000
EXPOSE 8080
ENTRYPOINT ["/usr/bin/docker-entrypoint.sh"]
CMD ["yarrbot"]