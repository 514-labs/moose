#!/bin/bash
# Helper script to set a GCP firewall rule
# This is necessary in order to allow traffic to enter the kubernetes cluster.
# This will create a firewall rule called ws-firewall that allows traffic on port 4000
# Example: ./set-firewall-rule.sh ws-firewall 4000

good=1
if [ -z "$1" ]
then
    echo "Missing parameter rule name"
    good=0
fi

if [ -z "$2" ]
then
    echo "Missing parameter port number"
    good=0
fi

if [ $good -eq 0 ]
then
    echo "Usage: ./set-firewall-rule.sh ws-firewall 4000"
    exit 1
fi

gcloud compute firewall-rules delete $1
gcloud compute firewall-rules create $1 --allow tcp:$2
