#/usr/bin/env bash

set -eo pipefail

# This script should be called from the root of the repository

version=$1

cd ./packages/ts-moose-lib
npm version $version --no-git-tag-version

# # This is run twice since the change the value of the dependencies in the previous step
pnpm install --filter "@514labs/moose-lib" --no-frozen-lockfile # requires optional dependencies to be present in the registry
pnpm build --filter @514labs/moose-lib

cd packages/ts-moose-lib
pnpm publish --access public --no-git-checks