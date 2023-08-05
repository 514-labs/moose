#/usr/bin/env bash

version=$1

if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+-BUILD\.[0-9]+$ ]]; then
   npm version $version --no-git-tag-version

   # change all the dependencies in the package.json optionalDependencies to use 
   #the BUILD version
   jq -r '.optionalDependencies | keys[]' package.json | while read dep; do
      npm install --save-optional "$dep@$version"
   done
fi

# pnpm install # requires optional dependencies to be present in the registry
# turbo build --filter @514labs/igloo-cli

# npm publish --access public