#/usr/bin/env bash

set -eo pipefail

version=$1

cd packages/event-capture
npm version $version --no-git-tag-version

jq \
    --arg VERSION "$version" \
    package.json > package.json.tmp \
    && mv package.json.tmp package.json

cd ../..
pnpm build --filter=@514labs/event-capture
cd packages/event-capture
pnpm publish --access public --no-git-checks
