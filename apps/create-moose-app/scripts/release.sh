#/usr/bin/env bash

version=$1

cd apps/create-moose-app
npm version $version --no-git-tag-version

jq \
    --arg VERSION "$version" \
    '.["dependencies"]["moose"] = $VERSION' package.json > package.json.tmp \
    && mv package.json.tmp package.json

cd ../..
pnpm build --filter create-moose-app
cd apps/create-moose-app
pnpm publish --access public --no-git-checks