#!/bin/bash

# Generates a version string for the current commit.
# if the current commit is tagged with a version, 
# then the version string is the tag name.
# Otherwise, the version string is the version in the package.json file
# with the build number appended
ref="$GITHUB_REF"
buildId="$GITHUB_RUN_NUMBER"

# Folder with a package.json file inside to read the version from
folder="$1"

currentVersion=$(cat .version)
if [[ "$ref" =~ ^refs/tags/[a-z\-]+([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
   echo "VERSION=${BASH_REMATCH[1]}"
else
   currentVersion=$(cat $folder/package.json | jq -r .version)
   if [[ "$currentVersion" =~ ^([0-9]+\.[0-9]+\.[0-9]+)\-BUILD\.[0-9]+$ ]]; then
      echo "VERSION=${BASH_REMATCH[1]}-BUILD.$buildId"
   else
      echo "VERSION=$currentVersion-BUILD.$buildId"
   fi
fi