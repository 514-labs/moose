#/usr/bin/env bash

set -eo pipefail

# This script should be called from the root of the repository

version=$1

cd ./packages/ts-moose-connectors
npm version $version --no-git-tag-version
cd ../..

# This is run twice since the change the value of the dependencies in the previous step
pnpm install --filter "@514labs/moose-connectors" --no-frozen-lockfile # requires optional dependencies to be present in the registry
pnpm build --filter @514labs/moose-connectors

cd packages/ts-moose-connectors
pnpm publish --access public --no-git-checks