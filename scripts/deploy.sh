#!/bin/sh
# Stages the service and its path dependency into one build context and deploys it on fly.io
# (remote builder, nothing compiles on this machine). Run from the repo root.
set -eu
S=$(mktemp -d)
mkdir -p "$S/airlock" "$S/signer/spikes"
cp -R Cargo.toml Cargo.lock src data Dockerfile fly.toml "$S/airlock/"
cp Dockerfile "$S/Dockerfile"
SIGNER=$(sed -n 's|^countersign-core = { path = "\.\./\([^/]*\)/.*|\1|p' Cargo.toml)
cp -R "../$SIGNER/Cargo.toml" "../$SIGNER/Cargo.lock" "../$SIGNER/crates" "$S/signer/"
cp -R "../$SIGNER/spikes/pczt-split" "$S/signer/spikes/"
sed -i '' "s|\.\./$SIGNER/|../signer/|" "$S/airlock/Cargo.toml"
cd "$S"
cp airlock/fly.toml fly.toml
flyctl deploy --app airlock-zec --remote-only
