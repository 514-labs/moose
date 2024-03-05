#!/bin/bash

version=$2

if [ -z "$1" ]
then
      echo "You must specify the dockerhub repository as an argument. Example: ./push.sh container-repo-name"
      echo "Note: you can also provide a second argument to supply a specific version tag - otherwise this script will use the same version as the latest moose-cli on Github."
      exit 1
fi

if [ -z "$2" ]
then
      output=$(npx @514labs/moose-cli -V)
      version=$(echo "$output" | sed -n '2p' | awk '{print $2}')
fi

echo "Using version: $version"
arch="moose-df-deployment-aarch64-unknown-linux-gnu"
docker tag $arch:$version $1/$arch:$version
docker push $1/$arch:$version

arch="moose-df-deployment-x86_64-unknown-linux-gnu"
docker tag $arch:$version $1/$arch:$version
docker push $1/$arch:$version
