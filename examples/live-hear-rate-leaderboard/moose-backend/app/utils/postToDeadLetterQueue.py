from moose_lib import Logger
import requests

ingest_endpoint = "http://localhost:4000/ingest/DeadLetterQueue"


def postToDeadLetterQueue(data: dict):
    logger = Logger(action="DLQ")
    logger.info(f"Posting data to dead letter queue: {data}")
    headers = {'Content-Type': 'application/json'}
    response = requests.post(ingest_endpoint, headers=headers, json=data)
    logger.info(f"Response from dead letter queue: {response.json()}")
