# Python Moose Lib

Python package which contains moose utils and streaming functionality.

## Features

- Data model definitions
- Streaming functions support
- Stream transformations (single and multi-destination)

## Streaming Functions

The library now supports streaming functions, allowing Python projects to define transformations between streams and automatically execute them. This is equivalent to the functionality already provided for TypeScript Moose projects.

### Basic Usage

```python
from pydantic import BaseModel
from moose_lib.dmv2 import Stream, StreamConfig

# Define your data models
class SourceData(BaseModel):
    name: str
    value: int

class DestinationData(BaseModel):
    processed_name: str
    processed_value: int

# Create streams
source_stream = Stream[SourceData](name="source_stream", config=StreamConfig())
dest_stream = Stream[DestinationData](name="dest_stream", config=StreamConfig())

# Define a transformation function
def transform_data(data: SourceData) -> DestinationData:
    return DestinationData(
        processed_name=f"processed_{data.name}",
        processed_value=data.value * 2
    )

# Add the transformation to the source stream
source_stream.add_transform(dest_stream, transform_data)
```

### Multi-Destination Transforms

For more complex scenarios, you can use multi-destination transforms:

```python
def multi_transform(data: SourceData):
    return [
        dest_stream1.routed(transform1(data)),
        dest_stream2.routed(transform2(data))
    ]

source_stream.set_multi_transform(multi_transform)
```

### Command Line Interface

The package includes a command-line interface for running streaming functions:

```bash
# List available streaming functions
moose-streaming list

# Run a streaming function
moose-streaming run --source-topic source_topic --target-topic dest_topic --transform-key source_stream_0_0_dest_stream_0_0

# Export streaming function information
moose-streaming export
```
