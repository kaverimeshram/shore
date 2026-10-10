#!/bin/sh
# Builds the Cloudflare Pages project "shore-zec" (shore.pages.dev was taken): the static web app (replay by default) plus one Pages
# Function, /store/*, the D1-backed state store the Rust API calls with a bearer token.
set -eu
cd "$(dirname "$0")"
rm -rf dist functions && mkdir -p dist functions/store
cp -R ../../web/shore ../../web/fonts ../../web/config.js dist/
rm -rf dist/shore/design-research dist/shore/.uicraft-read.json
printf '/ /shore/ 302\n' > dist/_redirects
cp ../shore-store/src/index.js functions/store/_store.js
printf 'import store from "./_store.js";\nexport const onRequest = (ctx) => store.fetch(ctx.request, ctx.env);\n' > 'functions/store/[[path]].js'
