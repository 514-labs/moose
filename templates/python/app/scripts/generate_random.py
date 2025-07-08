from moose_lib import Task, TaskConfig, Workflow, WorkflowConfig 
from datetime import datetime
from faker import Faker
from app.ingest.models import Foo, Baz
import requests

def run_task() -> None:
    fake = Faker()
    for i in range(1000):
        # Prepare request data
        foo = Foo(
            primary_key=fake.uuid4(),
            timestamp=fake.date_time_between(start_date='-1y', end_date='now').timestamp(),
            baz=fake.random_element(Baz),
            optional_text=fake.text() if fake.boolean() else None
        )
 
        # POST record to Moose Ingest API
        req = requests.post(
            "http://localhost:4000/ingest/Foo",
            data=foo.model_dump_json().encode('utf-8'),
            headers={'Content-Type': 'application/json'}
        )

ingest_task = Task[Foo, None](
    name="task",
    config=TaskConfig(run=run_task)
)

ingest_workflow = Workflow(
    name="workflow",
    config=WorkflowConfig(starting_task=ingest_task, schedule="@every 5s")
)
