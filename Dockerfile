# Context is the directory scripts/deploy.sh stages: airlock/ and the signing crate beside it.
# The service is stateless: it stores nothing. Tor consensus cache lives under /data and may be discarded.
FROM rust:1-bookworm AS build
RUN apt-get update && apt-get install -y --no-install-recommends clang protobuf-compiler libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY signer ./signer
COPY airlock ./airlock
WORKDIR /src/airlock
RUN cargo build --release --locked --bin airlock

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/airlock/target/release/airlock /usr/local/bin/airlock
WORKDIR /data
COPY airlock/data /data/data
EXPOSE 8787
CMD ["airlock", "serve", "0.0.0.0:8787"]
