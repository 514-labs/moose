from moose_lib import task  
from datetime import datetime
from faker import Faker
from app.ingest.models import Foo
import requests

@task
def generate_random():  # The name of your script
    """
    This is a sample task that generates random data for the `Foo` data model and sends it to the ingest API
    """
    fake = Faker()
    for i in range(1000):

        # Prepare request data
        foo = Foo(
            primary_key=fake.uuid4(),
            timestamp=fake.date_time_between(start_date='-1y', end_date='now').timestamp(),
            optional_text=fake.text() if fake.boolean() else None
        )
 
        # POST record to Moose Ingest API
        req = requests.post(
            "http://localhost:4000/ingest/Foo",
            data=foo.model_dump_json().encode('utf-8'),
            headers={'Content-Type': 'application/json'}
        )
        req.raise_for_status()
        
        
    ## Tasks must return a dictionary with the following keys:
    # "task": The name of the task function
    # "data": A dictionary of data to be returned to the workflow
    return {
      "task": "generate_random",
      "data": {
          "completed_at": datetime.now().isoformat()
      }
    }
    
    
