# Database Tables

## Overview
OLAP (Online Analytical Processing) tables in Moose provide a powerful way to store and query your data. They support high-performance analytics, real-time data ingestion, and efficient querying capabilities.

## Basic Table Setup

```python
from moose_lib import OlapTable, Key
from pydantic import BaseModel

class UserEvent(BaseModel):
    id: Key[str]
    user_id: str
    event_type: str
    timestamp: str

# Create a table
user_event_table = OlapTable(
    name="UserEventTable",
    order_by_fields=["id", "timestamp"],
    partition_by="toYYYYMM(timestamp)"
)
```

## Table Configuration

The `OlapTable` class accepts the following configuration:

```python
from typing import TypedDict, Optional, List

class TableConfig(TypedDict):
    name: str            # Required: Name of the table
    order_by_fields: List[str]  # Required: Fields to order by
    partition_by: Optional[str] = None  # Optional: Partition expression
    ttl: Optional[int] = None   # Optional: Time-to-live in seconds
    settings: Optional[dict] = None  # Optional: Table settings
```

## Table Operations

### Writing Data
```python
# Write a single record
await user_event_table.write({
    "id": "123",
    "user_id": "user_456",
    "event_type": "login",
    "timestamp": "2024-03-20T12:00:00Z"
})

# Write multiple records
await user_event_table.write_many([
    {
        "id": "123",
        "user_id": "user_456",
        "event_type": "login",
        "timestamp": "2024-03-20T12:00:00Z"
    },
    {
        "id": "124",
        "user_id": "user_457",
        "event_type": "logout",
        "timestamp": "2024-03-20T12:01:00Z"
    }
])
```

### Querying Data
```python
# Basic query
results = await user_event_table.query({
    "select": ["id", "user_id", "event_type"],
    "where": "event_type = 'login'",
    "limit": 10
})

# Advanced query with aggregations
stats = await user_event_table.query({
    "select": [
        "event_type",
        "count() as count",
        "min(timestamp) as first_seen",
        "max(timestamp) as last_seen"
    ],
    "group_by": ["event_type"],
    "order_by": ["count DESC"],
    "limit": 5
})
```

## Table Maintenance

### Partitioning
```python
# Create a table with partitioning
time_series_table = OlapTable(
    name="TimeSeriesTable",
    order_by_fields=["id", "timestamp"],
    partition_by="toYYYYMM(timestamp)"
)

# Get partition information
partitions = await time_series_table.get_partitions()
print("Table partitions:", partitions)

# Drop old partitions
await time_series_table.drop_partition("202401")
```

### TTL (Time-to-Live)
```python
# Create a table with TTL
temporary_table = OlapTable(
    name="TemporaryTable",
    order_by_fields=["id"],
    ttl=86400  # 24 hours in seconds
)
```

## Best Practices

1. **Table Design**
   - Choose appropriate primary keys
   - Use meaningful field names
   - Consider query patterns
   - Plan for data growth

2. **Partitioning**
   - Partition by time for time-series data
   - Use appropriate partition granularity
   - Monitor partition sizes
   - Clean up old partitions

3. **Performance**
   - Use appropriate indexes
   - Monitor query performance
   - Set appropriate TTLs
   - Use batch operations

4. **Maintenance**
   - Monitor table sizes
   - Clean up old data
   - Optimize table settings
   - Back up important data

## Example Usage

### Time Series Table
```python
from moose_lib import OlapTable, Key
from pydantic import BaseModel

class TimeSeriesEvent(BaseModel):
    id: Key[str]
    metric: str
    value: float
    timestamp: str

# Create table
metrics_table = OlapTable(
    name="MetricsTable",
    order_by_fields=["id", "timestamp"],
    partition_by="toYYYYMM(timestamp)",
    ttl=30 * 24 * 60 * 60  # 30 days
)

# Write metrics
await metrics_table.write({
    "id": "123",
    "metric": "cpu_usage",
    "value": 75.5,
    "timestamp": "2024-03-20T12:00:00Z"
})

# Query metrics
metrics = await metrics_table.query({
    "select": [
        "metric",
        "avg(value) as avg_value",
        "max(value) as max_value"
    ],
    "where": "timestamp >= '2024-03-20'",
    "group_by": ["metric"]
})
```

### Analytics Table
```python
from moose_lib import OlapTable, Key
from pydantic import BaseModel
from typing import Dict, Any

class AnalyticsEvent(BaseModel):
    id: Key[str]
    user_id: str
    action: str
    properties: Dict[str, Any]
    timestamp: str

# Create table
analytics_table = OlapTable(
    name="AnalyticsTable",
    order_by_fields=["id", "timestamp"],
    partition_by="toYYYYMM(timestamp)"
)

# Write analytics event
await analytics_table.write({
    "id": "123",
    "user_id": "user_456",
    "action": "page_view",
    "properties": {
        "page": "/home",
        "referrer": "google.com"
    },
    "timestamp": "2024-03-20T12:00:00Z"
})

# Query analytics
stats = await analytics_table.query({
    "select": [
        "action",
        "count() as count",
        "count(distinct user_id) as unique_users"
    ],
    "where": "timestamp >= '2024-03-20'",
    "group_by": ["action"],
    "order_by": ["count DESC"]
})
``` 