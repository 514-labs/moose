from app.ingest.models import fooModel, barModel, Bar
from datetime import datetime

# Transform Foo events to Bar events
fooModel.get_stream().add_transform(
    destination=barModel.get_stream(),
    transformation=lambda foo: Bar(
            primary_key=foo.primary_key,
            utc_timestamp=datetime.fromtimestamp(foo.timestamp),
            has_text=foo.optional_text is not None,
            text_length=len(foo.optional_text) if foo.optional_text else 0
        )
    )

# Add a streaming consumer to print Foo events
def print_foo_event(foo):
    print(f"Received Foo event:")
    print(f"  Primary Key: {foo.primary_key}")
    print(f"  Timestamp: {datetime.fromtimestamp(foo.timestamp)}")
    print(f"  Optional Text: {foo.optional_text or 'None'}")
    print("---")

fooModel.get_stream().add_consumer(print_foo_event)


