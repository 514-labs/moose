#/usr/bin/env bash

set -eo pipefail

version=$1

cd packages/design-system
npm version $version --no-git-tag-version

jq \
    --arg VERSION "$version" \
    package.json > package.json.tmp \
    && mv package.json.tmp package.json

jq '.dependencies["@514labs/event-capture"]="$version"' \
    package.json > package.json.tmp \
    && mv package.json.tmp package.json

cd ../..
pnpm build --filter=design-system
cd packages/design-system
pnpm publish --access public --no-git-checks
