#/usr/bin/env bash

set -eo pipefail

# This script should be called from the root of the repository

version=$1

cd ./packages/ts-connector-s3
npm version $version --no-git-tag-version

# Update the peer dependency version to match this package version
# Use platform-specific sed command
if [[ "$OSTYPE" == "darwin"* ]]; then
  # macOS (BSD sed)
  sed -i '' "s/\"@514labs\/moose-lib\": \"\*\"/\"@514labs\/moose-lib\": \"\^$version\"/" package.json
else
  # Linux (GNU sed)
  sed -i "s/\"@514labs\/moose-lib\": \"\*\"/\"@514labs\/moose-lib\": \"\^$version\"/" package.json
fi

cd ../..

# This is run twice since the change the value of the dependencies in the previous step
pnpm install --filter "@514labs/moose-connector-s3" --no-frozen-lockfile # requires optional dependencies to be present in the registry
pnpm build --filter @514labs/moose-connector-s3

cd packages/ts-connector-s3
pnpm publish --access public --no-git-checks