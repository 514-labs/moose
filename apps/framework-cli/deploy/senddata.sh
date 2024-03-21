#!/bin/bash

re='^[0-9]+$'

if [[ $1 =~ $re ]]; then
    count=$1
    endpoint="http://localhost:4000/ingest/UserActivity/"
else
    endpoint=$1
    count=$2
fi

if [ -z "$endpoint" ]
then
    endpoint="http://localhost:4000/ingest/UserActivity/"
fi

if [ -z "$count" ]
then
    count=1
fi

for (( i=1; i<=$count; i++ ))
do
    eventID=$(LC_ALL=C < /dev/urandom tr -dc 'a-zA-Z0-9' | fold -w 10 | head -n 1)
    userID=$(LC_ALL=C < /dev/urandom tr -dc 'a-zA-Z0-9' | fold -w 6 | head -n 1)
    currentTimestamp=$(date '+%Y-%m-%d %H:%M:%S')

    curl -v -X POST \
    -H "Content-Type: application/json" \
    -d "{\"eventId\": \"$eventID\", \"timestamp\": \"$currentTimestamp\", \"userId\": \"$userID\", \"activity\": \"click\"}" \
    $endpoint
done