# docker build --build-context countersign=../countersign -t airlock .
# Builds the AIRLOCK service. It is stateless: it stores nothing, so no volume is needed.
# Tor state (consensus cache) lives under /data and may be discarded.
FROM rust:1-bookworm AS build
WORKDIR /src
COPY . .
# countersign-core is a path dependency of this workspace
COPY --from=countersign . /countersign
RUN cargo build --release --bin airlock

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/airlock /usr/local/bin/airlock
WORKDIR /data
EXPOSE 8787
CMD ["airlock", "serve", "0.0.0.0:8787"]
