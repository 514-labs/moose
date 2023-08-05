#/usr/bin/env bash

version=$1

if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+-BUILD\.[0-9]+$ ]]; then
   npm version $version --no-git-tag-version

   # change all the dependencies in the package.json optionalDependencies to use 
   #the BUILD version
   jq -r '.optionalDependencies | keys[]' package.json | while read dep; do
    #   pnpm up $dep $version
      jq --arg DEP "$dep" --arg VERSION "$version" '.["optionalDependencies"][$DEP] = $VERSION' package.json > package.json.tmp && mv package.json.tmp package.json
   done
fi

pnpm install # requires optional dependencies to be present in the registry
turbo build --filter @514labs/igloo-cli

pnpm publish --access public