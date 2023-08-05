#/usr/bin/env bash

cd apps/igloo-kit-cli

pnpm install # requires optional dependencies to be present in the registry
pnpm build

npm version 
npm publish --access public