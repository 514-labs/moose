#!/bin/bash
# Helper script to set a secret in the kubernetes cluster for pulling private docker images
# Example: ./set-dockerhub-credentials.sh cjus514 fakepassword carlos@fiveonefour.com
# TODO: Update this script to use a 514 service account
good=1
if [ -z "$1" ]
then
    echo "Missing parameter Docker_REGISTRY_USER"
    good=0
fi

if [ -z "$2" ]
then
    echo "Missing parameter Docker_REGISTRY_PASSWORD"
    good=0
fi

if [ -z "$2" ]
then
    echo "Missing parameter Docker_REGISTRY_EMAIL"
    good=0
fi

if [ $good -eq 0 ]
then
    echo "Usage: ./set-dockerhub-credentials.sh cjus514 fakepassword carlos@fiveonefour.com"
    exit 1
fi

kubectl create secret docker-registry moose-repo-credentials \
--docker-server=https://index.docker.io/v1/ \
--docker-username=$1 \
--docker-password=$2 \
--docker-email=$3
