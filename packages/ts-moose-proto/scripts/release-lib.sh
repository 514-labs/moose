#/usr/bin/env bash

set -eo pipefail

# This script should be called from the root of the repository

version=$1

cd ./packages/ts-moose-proto
npm version $version --no-git-tag-version
cd ../..

pnpm install --filter "@514labs/moose-proto" --frozen-lockfile
pnpm --filter @514labs/moose-proto run gen
pnpm --filter @514labs/moose-proto run build

cd packages/ts-moose-proto
pnpm publish --access public --no-git-checks
