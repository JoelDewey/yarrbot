FROM rust:1.53.0 as builder

WORKDIR /usr/src/myapp

COPY ./ ./

RUN apt-get update && apt-get install -y cmake libpq-dev
RUN cargo build --release
RUN strip ./target/release/yarrbot

FROM ubuntu:latest

RUN apt-get update && apt-get install -y libpq5 openssl ca-certificates && rm -rf /var/lib/apt/lists/*
RUN addgroup --system --gid 1000 yarrbot && adduser --system --no-create-home --shell /bin/false --uid 1000 --gid 1000 yarrbot
COPY --chown=yarrbot:yarrbot --from=builder /usr/src/myapp/target/release/yarrbot /app/yarrbot
RUN chown -R yarrbot:yarrbot /app/yarrbot && mkdir /data && chown yarrbot:yarrbot /data

VOLUME ["/data"]

USER yarrbot
ENV YARRBOT_STORAGE_DIR=/data
EXPOSE 8080
ENTRYPOINT ["/app/yarrbot"]