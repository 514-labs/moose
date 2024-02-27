#!/bin/bash

mkdir -p deployment
cp -R app ./deployment

cp /Users/cjus/dev/moose/apps/framework-cli/deploy/pseudobuild.sh .
cp /Users/cjus/dev/moose/apps/framework-cli/deploy/buildx_setup.sh .
cp /Users/cjus/dev/moose/apps/framework-cli/deploy/Dockerfile.deployment ./deployment/Dockerfile
