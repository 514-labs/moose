from moose_lib import task  
import random
import time
import uuid
import json
import urllib.request
from datetime import datetime

@task
def generate_random():  # The name of your script
    """
    This is a sample task that generates random data for the `Foo` data model and sends it to the ingest API
    """
    for i in range(1000):
        # Generate random data
        primary_key = str(uuid.uuid4())
        # Generate random timestamp from last month
        current_time = time.time()
        one_month_ago = current_time - (30 * 24 * 60 * 60)  # 30 days in seconds
        timestamp = random.uniform(one_month_ago, current_time)
        
        # Generate random optional text
        words = [word.strip() for word in open('/usr/share/dict/words').readlines()]  # Standard Unix word list
        optional_text = f"{random.choice(words)} {i}" if i % 2 == 0 else None
 
        # Prepare request data
        data = {
            "primary_key": primary_key,
            "timestamp": timestamp,
            "optional_text": optional_text
        }
 
        # Send POST request using only stdlib
        req = urllib.request.Request(
            "http://localhost:4000/ingest/Foo",
            data=json.dumps(data).encode('utf-8'),
            headers={'Content-Type': 'application/json'}
        )
        urllib.request.urlopen(req)
    ## Tasks must return a dictionary with the following keys:
    # "task": The name of the task function
    # "data": A dictionary of data to be returned to the workflow
    return {
      "task": "generate_random",
      "data": {
          "completed_at": datetime.now().isoformat()
      }
    }
    
    
