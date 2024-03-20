#!/bin/bash

eventID=$(LC_ALL=C < /dev/urandom tr -dc 'a-zA-Z0-9' | fold -w 10 | head -n 1)
userID=$(LC_ALL=C < /dev/urandom tr -dc 'a-zA-Z0-9' | fold -w 6 | head -n 1)
currentTimestamp=$(date '+%Y-%m-%d %H:%M:%S')

endpoint=$1
if [ -z "$1" ]
then
    endpoint="http://localhost:4000/ingest/UserActivity"
fi

curl -v -X POST \
-H "Content-Type: application/json" \
-d "{\"eventId\": \"$eventID\", \"timestamp\": \"$currentTimestamp\", \"userId\": \"$userID\", \"activity\": \"click\"}" \
$endpoint

