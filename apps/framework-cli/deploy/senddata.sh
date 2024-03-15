#!/bin/bash

eventID=$(LC_ALL=C < /dev/urandom tr -dc 'a-zA-Z0-9' | fold -w 10 | head -n 1)
userID=$(LC_ALL=C < /dev/urandom tr -dc 'a-zA-Z0-9' | fold -w 6 | head -n 1)
currentTimestamp=$(date '+%Y-%m-%d %H:%M:%S')

curl -v -X POST \
-H "Content-Type: application/json" \
-d "{\"eventId\": \"$eventID\", \"timestamp\": \"$currentTimestamp\", \"userId\": \"$userID\", \"activityType\": \"click\"}" \
http://localhost:4000/ingest/UserActivity
