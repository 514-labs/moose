#/usr/bin/env bash

# This script should be called from the root of the repository

version=$1

if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+-BUILD\.[0-9]+$ ]]; then
   cd apps/igloo-kit-cli
   npm version $version --no-git-tag-version

   # change all the dependencies in the package.json optionalDependencies to use 
   # the BUILD version
   jq -r '.optionalDependencies | keys[]' package.json | while read dep; do
    #   pnpm up $dep $version
      jq \
        --arg DEP "$dep" \
        --arg VERSION "$version" \
        '.["optionalDependencies"][$DEP] = $VERSION' package.json > package.json.tmp \
        && mv package.json.tmp package.json
   done
   cd ../..
fi

# # This is run twice since the change the value of the dependencies in the previous step
pnpm install --no-frozen-lockfile # requires optional dependencies to be present in the registry
pnpm build --filter @514labs/igloo-cli

cd apps/igloo-kit-cli
pnpm publish --access public --no-git-checks