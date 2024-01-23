#/usr/bin/env bash
set -eo pipefail

version=$1

cd apps/create-igloo-app
npm version $version --no-git-tag-version

jq \
    --arg VERSION "$version" \
    '.["dependencies"]["@514labs/igloo-cli"] = $VERSION' package.json > package.json.tmp \
    && mv package.json.tmp package.json

cd ../..
pnpm build --filter create-igloo-app
cd apps/create-igloo-app
pnpm publish --access public --no-git-checks