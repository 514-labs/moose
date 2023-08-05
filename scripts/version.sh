#!/bin/bash

# Generates a version string for the current commit.
# if the current commit is tagged with a version, 
# then the version string is the tag name.
ref="$GITHUB_REF"
echo $ref
if [[ "$ref" =~ ^refs/tags/[a-z\-]+([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
   echo "${BASH_REMATCH[1]}"
else
   echo "Out"
fi