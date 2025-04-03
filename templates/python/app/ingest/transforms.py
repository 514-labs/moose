from app.ingest.models import fooModel, barModel, Bar
from datetime import datetime

fooModel.get_stream().add_transform(
    destination=barModel.get_stream(),
    transformation=lambda foo: Bar(
            primary_key=foo.primary_key,
            utc_timestamp=datetime.fromtimestamp(foo.timestamp),
            has_text=foo.optional_text is not None,
            text_length=len(foo.optional_text) if foo.optional_text else 0
        )
    )


