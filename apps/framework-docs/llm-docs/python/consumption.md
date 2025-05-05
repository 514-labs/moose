# Consumption APIs

## Overview
Consumption APIs make it easy to build type-safe HTTP `GET` endpoints for surfacing data from your OLAP database. These APIs can help power user-facing analytics, dashboards and other front-end components, or even enable AI tools to interact with your data.

## Creating API Endpoints

```python
from moose_lib.dmv2 import ConsumptionApi
from moose_lib import MooseClient
from pydantic import BaseModel, Field
from typing import Optional

# Define the query parameters
class QueryParams(BaseModel):
    filterField: str
    maxResults: int = Field(default=10, gt=0, le=100)
    optionalParam: Optional[str] = None

# Define the response body
class ResponseBody(BaseModel):
    id: int
    name: str

# Define the route handler function
def run(client: MooseClient, params: QueryParams) -> ResponseBody:
    query = """
    SELECT id, name 
    FROM example_table 
    WHERE filter_field = {filter_field} 
    LIMIT {max_results}
    """
    # Always use parameterized queries for security
    return client.query.execute(
        query, 
        {
            "filter_field": params.filterField,
            "max_results": params.maxResults
        }
    )

# Create the API
example_api = ConsumptionApi[QueryParams, ResponseBody](
    name="example_endpoint",
    query_function=run
)
```

## Defining Query Parameters

Define your API's parameters as a Pydantic model:

```python
from pydantic import BaseModel

class QueryParams(BaseModel):
    filterField: str
    maxResults: int
    optionalParam: str | None = None
```

Moose automatically handles:
- Runtime validation
- Clear error messages for invalid parameters
- OpenAPI documentation generation

Complex nested objects and arrays are not supported. Consumption APIs are `GET` endpoints designed to be simple and lightweight.

### Adding Advanced Type Validation

Moose uses Pydantic for runtime validation. Use Pydantic's `Field` class for more complex validation:

```python
from pydantic import BaseModel, Field

class QueryParams(BaseModel):
    filterField: str
    maxResults: int = Field(..., gt=0)
```

### Common Validation Options

```python
from pydantic import BaseModel, Field

class QueryParams(BaseModel):
    # Numeric validations
    id: int = Field(..., gt=0)
    age: int = Field(..., gt=0, lt=120)
    price: float = Field(..., gt=0, lt=1000)
    discount: float = Field(..., gt=0, multiple_of=0.5)

    # String validations
    username: str = Field(..., min_length=3, max_length=20)
    email: str = Field(..., format="email")
    zipCode: str = Field(..., pattern=r"^[0-9]{5}$")
    uuid: str = Field(..., format="uuid")
    ipAddress: str = Field(..., format="ipv4")

    # Date validations
    startDate: str = Field(..., format="date")

    # Enum validation
    status: str = Field(..., enum=["active", "pending", "inactive"])

    # Optional parameters
    limit: int = Field(None, gt=0, lt=100)
```

For a full list of validation options, see the [Pydantic documentation](https://docs.pydantic.dev/latest/concepts/types/#customizing-validation-with-fields).

### Setting Default Values

You can set default values for parameters by setting values for each parameter in your Pydantic model:

```python
from pydantic import BaseModel

class QueryParams(BaseModel):
    filterField: str = "example"
    maxResults: int = 10
    optionalParam: str | None = "default"
```

## Response Type Requirements

The response type for a Consumption API must be a single Pydantic model that directly extends `BaseModel`. You cannot wrap the response type in containers like `List`, `Dict`, or other collection types.

✅ Correct response type:
```python
class UserStats(BaseModel):
    user_id: int
    total_actions: int
    last_active: str

# This is correct - UserStats directly extends BaseModel
user_stats_api = ConsumptionApi[QueryParams, UserStats](...)
```

❌ Incorrect response types:
```python
# Wrong - Cannot wrap UserStats in List
user_stats_api = ConsumptionApi[QueryParams, List[UserStats]](...)

# Wrong - Cannot wrap UserStats in Dict
user_stats_api = ConsumptionApi[QueryParams, Dict[str, UserStats]](...)

# Wrong - Cannot use primitive types directly
user_stats_api = ConsumptionApi[QueryParams, str](...)
```

If you need to return multiple items, include them as a field in your response model:

```python
class UserStatsResponse(BaseModel):
    items: List[UserStats]  # Correct - the collection is a field inside the model
    total_count: int

# This is correct - UserStatsResponse directly extends BaseModel
user_stats_api = ConsumptionApi[QueryParams, UserStatsResponse](...)
```

## Implementing Route Handler

Here's a complete example showing how to implement a route handler with proper query parameterization:

```python
from moose_lib import MooseClient
from moose_lib.dmv2 import ConsumptionApi
from pydantic import BaseModel, Field
from typing import Optional
from app.views.user_stats import userStatsMV

class QueryParams(BaseModel):
    start_date: str = Field(..., description="Start date in YYYY-MM-DD format")
    end_date: str = Field(..., description="End date in YYYY-MM-DD format")
    limit: Optional[int] = Field(
        default=10,
        gt=0,
        le=100,
        description="Number of results to return (1-100)"
    )

class UserStats(BaseModel):
    user_id: int
    total_actions: int
    last_active: str

def run(client: MooseClient, params: QueryParams) -> UserStats:
    query = f"""
    SELECT 
        user_id,
        count() as total_actions,
        max(activity_date) as last_active
    FROM {userStatsMV.target_table.name}
    WHERE activity_date >= {{start_date}}
    AND activity_date <= {{end_date}}
    GROUP BY user_id
    ORDER BY total_actions DESC
    LIMIT {{limit}}
    """
    
    # Always use parameterized queries to prevent SQL injection
    return client.query.execute(query, {
        "start_date": params.start_date,
        "end_date": params.end_date,
        "limit": params.limit
    })

user_stats_api = ConsumptionApi[QueryParams, UserStats](
    name="user_stats",
    query_function=run
)
```

### Advanced Query Patterns

```python
from app.tables import myTable

# Example with dynamic column selection
def run(client: MooseClient, params: QueryParams) -> dict:
    col_name = "id"
    query = f"""
    SELECT {myTable.columns[col_name]} 
    FROM {myTable.name}
    WHERE status = {{status}}
    LIMIT {{limit}}
    """
    
    return client.query.execute(query, {
        "status": params.status,
        "limit": params.limit
    })
```

### Adding Authentication

```python
def run(params: QueryParams, client: MooseClient, jwt: dict) -> None:
    query = f"""SELECT * FROM userReports WHERE user_id = {jwt["userId"]} LIMIT {limit}"""
    data = client.query.execute(query)
    return data
```

Moose validates the JWT signature and ensures the JWT is properly formatted. If the JWT authentication fails, Moose will return a `401 Unauthorized error`.

## Understanding Response Codes

Moose automatically provides standard HTTP responses:

| Status Code | Meaning | Response Body |
|-------------|-------------------------|---------------------------------|
| 200 | Success | Your API's result data |
| 400 | Validation error | `{ "error": "Detailed message"}` |
| 401 | Unauthorized | `{ "error": "Unauthorized"}` |
| 500 | Internal server error | `{ "error": "Internal server error"}` |
