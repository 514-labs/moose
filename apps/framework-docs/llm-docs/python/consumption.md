# Consumption APIs

## Overview
Moose provides two main types of APIs for exposing data:

1. **EgressApi**: Use this for simple data surfacing from tables with SQL queries and parameter support. This is the recommended approach for most data access patterns.

2. **ConsumptionApi**: Use this for complex data transformations, custom business logic, or when you need fine-grained control over the response format.

Choose the right API type based on your needs:
- Use `EgressApi` when you just need to surface table data with filtering and pagination
- Use `ConsumptionApi` when you need custom business logic or complex data transformations

## EgressApi Usage

The `EgressApi` is the recommended way to surface table data. It provides a simple, type-safe interface for exposing data with built-in parameter support.

### Basic Setup
```python
from moose_lib import EgressApi

# Define an egress API for brain data
get_brain_data = EgressApi(
    name="get_brain_data",
    table="brain_data",
    sql="""
        SELECT * FROM brain_data
        WHERE ({start_time:Nullable(Float64)} IS NULL OR timestamp >= {start_time:Nullable(Float64)})
          AND ({end_time:Nullable(Float64)} IS NULL OR timestamp <= {end_time:Nullable(Float64)})
        ORDER BY timestamp DESC
        LIMIT {limit:Int32}
    """,
    query_params={
        "limit": {"type": "int", "default": 100},
        "start_time": {"type": "float", "required": False},
        "end_time": {"type": "float", "required": False},
    }
)
```

### Features
- Type-safe parameter handling
- Built-in pagination support
- Automatic parameter validation
- SQL query parameterization
- Support for nullable parameters
- Default parameter values

### Best Practices
1. **Parameter Types**
   - Use `Nullable(T)` for optional parameters
   - Specify exact types (e.g., `Float64`, `Int32`)
   - Provide default values when appropriate

2. **SQL Queries**
   - Use parameterized queries for all user input
   - Include proper WHERE clauses for filtering
   - Add ORDER BY for consistent results
   - Use LIMIT for pagination

3. **Error Handling**
   - The API handles parameter validation automatically
   - SQL errors are caught and returned as proper HTTP errors
   - Invalid parameter types are rejected

## ConsumptionApi Usage

The `ConsumptionApi` class requires both input (query) and output (response) type parameters, and must be instantiated with a name as the first positional parameter. Here's the correct pattern:

```python
from moose_lib import ConsumptionApi, EgressConfig
from pydantic import BaseModel
from typing import Dict, Any, Optional

# Define your query parameters model
class BrainDataQuery(BaseModel):
    id: str
    start_time: Optional[float] = None
    end_time: Optional[float] = None
    limit: Optional[int] = 100

# Define your response model
class BrainData(BaseModel):
    id: str
    timestamp: float
    value: float

# Define your query function
def get_brain_data(params: BrainDataQuery, utils) -> BrainData:
    client = utils.client
    sql = utils.sql
    
    result = client.query.execute("""
        SELECT * FROM brain_data
        WHERE id = {params.id}
        AND ({params.start_time} IS NULL OR timestamp >= {params.start_time})
        AND ({params.end_time} IS NULL OR timestamp <= {params.end_time})
        ORDER BY timestamp DESC
        LIMIT {params.limit}
    """)
    
    if not result:
        return BrainData(
            id="",
            timestamp=0.0,
            value=0.0
        )
    
    return BrainData(**result[0])

# CORRECT: Use generic type parameters, provide name as first positional parameter, and use EgressConfig
brain_data_api = ConsumptionApi[BrainDataQuery, BrainData](
    "get_brain_data",           # Required: First positional parameter
    query_function=get_brain_data,  # Required: Query function
    source="brain_data",        # Required: Source table name
    config=EgressConfig(        # Required: Configuration wrapped in EgressConfig
        auth={
            "required": True,
            "roles": ["admin"]
        }
    )
)

# INCORRECT: Missing name as first positional parameter
brain_data_api = ConsumptionApi[BrainDataQuery, BrainData](  # This will cause an error
    query_function=get_brain_data,
    source="brain_data",
    config=EgressConfig(
        auth={"required": True}
    )
)

# INCORRECT: Settings not wrapped in EgressConfig
brain_data_api = ConsumptionApi[BrainDataQuery, BrainData](  # This will cause an error
    "get_brain_data",
    query_function=get_brain_data,
    source="brain_data",
    settings={  # Wrong: settings should be in EgressConfig
        "auth": {"required": True}
    }
)
```

### Required Parameters

1. **Name Parameter**
   - Must be provided as the first positional parameter
   - Should match the API's purpose
   - Used for endpoint identification
   ```python
   # CORRECT
   api = ConsumptionApi[InputType, OutputType](
       "get_brain_data",  # Required: First positional parameter
       query_function=get_brain_data,
       source="brain_data",
       config=EgressConfig(...)
   )

   # INCORRECT
   api = ConsumptionApi[InputType, OutputType](  # Missing name as first positional parameter
       query_function=get_brain_data,
       source="brain_data",
       config=EgressConfig(...)
   )
   ```

2. **Configuration**
   - Must use `EgressConfig` for settings
   - Must provide required configuration options
   - Must be passed as `config` parameter
   ```python
   # CORRECT
   api = ConsumptionApi[InputType, OutputType](
       "get_brain_data",
       query_function=get_brain_data,
       source="brain_data",
       config=EgressConfig(
           auth={"required": True}
       )
   )

   # INCORRECT
   api = ConsumptionApi[InputType, OutputType](  # Settings not in EgressConfig
       "get_brain_data",
       query_function=get_brain_data,
       source="brain_data",
       settings={  # Wrong: should be in EgressConfig
           "auth": {"required": True}
       }
   )
   ```

### Common Issues and Solutions

1. **Missing Name Parameter**
   - Problem: Not providing name as first positional parameter
   - Solution: Always include name as first argument
   ```python
   # Wrong
   api = ConsumptionApi[BrainDataQuery, BrainData](
       query_function=get_brain_data,
       source="brain_data",
       config=EgressConfig(...)
   )
   
   # Correct
   api = ConsumptionApi[BrainDataQuery, BrainData](
       "get_brain_data",  # First positional parameter
       query_function=get_brain_data,
       source="brain_data",
       config=EgressConfig(...)
   )
   ```

2. **Incorrect Configuration**
   - Problem: Settings not wrapped in EgressConfig
   - Solution: Always use EgressConfig for settings
   ```python
   # Wrong
   api = ConsumptionApi[BrainDataQuery, BrainData](
       "get_brain_data",
       query_function=get_brain_data,
       source="brain_data",
       settings={  # Wrong: should be in EgressConfig
           "auth": {"required": True}
       }
   )
   
   # Correct
   api = ConsumptionApi[BrainDataQuery, BrainData](
       "get_brain_data",
       query_function=get_brain_data,
       source="brain_data",
       config=EgressConfig(  # Correct: settings in EgressConfig
           auth={"required": True}
       )
   )
   ```

### Best Practices

1. **API Naming**
   - Use descriptive names that match the API's purpose
   - Follow consistent naming conventions
   - Avoid generic names

2. **Configuration**
   - Always use EgressConfig for settings
   - Provide all required configuration options
   - Use proper authentication settings

3. **Type Safety**
   - Always define both input and output types
   - Use Pydantic models for type validation
   - Make query parameters explicit with a dedicated model

4. **Required Parameters**
   - Always provide name as first positional parameter
   - Always provide both type parameters
   - Always provide the query function
   - Always provide the source
   - Always use EgressConfig for settings

## Python-Specific Requirements

When creating Python consumption APIs, you must follow these requirements exactly:

1. **Imports**
   ```python
   from moose_lib import ConsumptionApi
   from pydantic import BaseModel
   from typing import Optional, Dict, Any
   ```

2. **Response Models**
   - Always use Pydantic models
   - Use standard Python types (not Key[str])
   - Use snake_case for field names
   - Define all nested models explicitly
   ```python
   class Accelerometer(BaseModel):
       x: float = 0.0
       y: float = 0.0
       z: float = 0.0

   class Gyroscope(BaseModel):
       x: float = 0.0
       y: float = 0.0
       z: float = 0.0

   class PPM(BaseModel):
       value: float = 0.0

   class BrainData(BaseModel):
       id: str
       timestamp: float
       acc: Accelerometer
       gyro: Gyroscope
       ppm: PPM
   ```

3. **Query Functions**
   - Must be synchronous (not async)
   - Must take params and utils arguments
   - Must return a single instance of the response model
   - Must never return None, a list, or a dict
   - Must handle the case when no data is found
   - Use snake_case for function names
   ```python
   # CORRECT: Returns a single model instance with default values
   def get_brain_data(params: Dict[str, Any], utils) -> BrainData:
       client = utils.client
       sql = utils.sql
       
       result = client.query.execute(sql"""
           SELECT *
           FROM brain_data
           WHERE id = {params['id']}
           LIMIT 1
       """)
       
       if not result:
           # Return a default-constructed model with zero values
           return BrainData(
               id="",
               timestamp=0.0,
               acc=Accelerometer(),
               gyro=Gyroscope(),
               ppm=PPM()
           )
       
       return BrainData(**result[0])

   # CORRECT: Returns a single model instance with custom defaults
   def get_user_event(params: Dict[str, Any], utils) -> UserEvent:
       client = utils.client
       sql = utils.sql
       
       result = client.query.execute(sql"""
           SELECT *
           FROM UserEventTable
           WHERE user_id = {params['user_id']}
           LIMIT 1
       """)
       
       if not result:
           # Return a default-constructed model with meaningful defaults
           return UserEvent(
               id="",
               user_id=params['user_id'],
               event_type="unknown",
               timestamp="1970-01-01T00:00:00Z"
           )
       
       return UserEvent(**result[0])

   # INCORRECT: Returns a list
   def get_brain_data(params: Dict[str, Any], utils) -> list[BrainData]:  # This will cause an error
       client = utils.client
       sql = utils.sql
       
       result = client.query.execute(sql"""
           SELECT * FROM brain_data
       """)
       
       return result  # Wrong: returning a list

   # INCORRECT: Returns None
   def get_brain_data(params: Dict[str, Any], utils) -> BrainData:
       client = utils.client
       sql = utils.sql
       
       result = client.query.execute(sql"""
           SELECT * FROM brain_data
           WHERE id = {params['id']}
           LIMIT 1
       """)
       
       if not result:
           return None  # Wrong: returning None
       
       return BrainData(**result[0])

   # INCORRECT: Returns a dict
   def get_brain_data(params: Dict[str, Any], utils) -> BrainData:
       client = utils.client
       sql = utils.sql
       
       result = client.query.execute(sql"""
           SELECT * FROM brain_data
           WHERE id = {params['id']}
           LIMIT 1
       """)
       
       if not result:
           return {}  # Wrong: returning a dict
       
       return result[0]  # Wrong: returning a dict
   ```

4. **API Creation**
   - Use two type parameters: response model and query function
   - The API name is derived from the variable name, DO NOT specify it in the constructor
   - Configure settings for auth and rate limiting
   ```python
   # CORRECT: API name is derived from the variable name
   BrainDataApi = ConsumptionApi[BrainData, get_brain_data](
       source="brain_data",
       settings={
           "auth": {
               "required": True,
               "roles": ["admin"]
           }
       }
   )

   # INCORRECT: Do not specify name parameter
   BrainDataApi = ConsumptionApi[BrainData, get_brain_data](
       name="BrainDataApi",  # This will cause an error
       source="brain_data"
   )
   ```

5. **Best Practices**
   - Use snake_case for all names
   - Use proper Python type hints
   - Never use dictionary expressions in type annotations
   - Always inherit from BaseModel for response models
   - Use parameterized SQL queries
   - Handle errors appropriately
   - Let the API name be derived from the variable name
   - Use descriptive variable names that match the API's purpose
   - Use synchronous functions (no async/await)
   - Always return a valid model instance (never None, list, or dict)
   - Provide meaningful default values for all fields
   - Handle the case when no data is found
   - Use proper type hints for all parameters and return values

## Common Issues and Solutions

### 1. Return Type Errors
- **Problem**: Function returns a list, None, or dict instead of a model instance
- **Solution**: Always return a valid model instance, even when no data is found
- **Example**:
  ```python
  # Wrong
  if not result:
      return None  # or return [] or return {}
  
  # Correct
  if not result:
      return BrainData(
          id="",
          timestamp=0.0,
          acc=Accelerometer(),
          gyro=Gyroscope(),
          ppm=PPM()
      )
  ```

### 2. Nested Model Errors
- **Problem**: Missing or invalid nested model definitions
- **Solution**: Define all nested models explicitly with proper defaults
- **Example**:
  ```python
  # Wrong
  class BrainData(BaseModel):
      id: str
      acc: Dict[str, float]  # Using dict instead of proper model
  
  # Correct
  class Accelerometer(BaseModel):
      x: float = 0.0
      y: float = 0.0
      z: float = 0.0
  
  class BrainData(BaseModel):
      id: str
      acc: Accelerometer
  ```

### 3. Type Hint Errors
- **Problem**: Incorrect return type hints
- **Solution**: Use proper type hints for the model and function
- **Example**:
  ```python
  # Wrong
  def get_brain_data(params: Dict[str, Any], utils) -> list[BrainData]:
  
  # Correct
  def get_brain_data(params: Dict[str, Any], utils) -> BrainData:
  ```

## Basic API Setup

```python
from moose_lib import ConsumptionApi
from pydantic import BaseModel
from typing import Optional, Dict, Any

class UserEvent(BaseModel):
    id: str
    user_id: str
    event_type: str
    timestamp: str

# Create a consumption API with a query function
def get_user_events(params: Dict[str, Any], utils) -> list[UserEvent]:
    client = utils.client
    sql = utils.sql
    
    # Use the SQL helper for type-safe queries
    result = await client.query.execute(sql"""
        SELECT *
        FROM UserEventTable
        WHERE user_id = {params['user_id']}
        ORDER BY timestamp DESC
        LIMIT {params.get('limit', 10)}
    """)
    
    return result

UserEventApi = ConsumptionApi[UserEvent, get_user_events](
    source="UserEventTable"
)
```

## API Configuration

The `ConsumptionApi` class accepts the following configuration:

```python
from typing import TypedDict, Literal, Optional, Callable, Any

class ApiConfig(TypedDict):
    name: str            # Required: Name of the API
    source: str          # Required: Source table or view
    settings: Optional[dict]  # Additional settings

# Query function signature
QueryFunction = Callable[[Dict[str, Any], Any], list[BaseModel]]
```

## API Operations

### Basic Endpoints
```python
# Get by ID endpoint
def get_event_by_id(params: Dict[str, Any], utils) -> Optional[UserEvent]:
    client = utils.client
    sql = utils.sql
    
    result = await client.query.execute(sql"""
        SELECT *
        FROM UserEventTable
        WHERE id = {params['id']}
        LIMIT 1
    """)
    
    return result[0] if result else None

GetByIdApi = ConsumptionApi[UserEvent, get_event_by_id](
    name="GetByIdApi",
    source="UserEventTable"
)

# List endpoint with filtering
def list_events(params: Dict[str, Any], utils) -> list[UserEvent]:
    client = utils.client
    sql = utils.sql
    
    result = await client.query.execute(sql"""
        SELECT *
        FROM UserEventTable
        WHERE user_id = {params['user_id']}
        AND timestamp >= {params['start_date']}
        AND timestamp <= {params['end_date']}
        ORDER BY timestamp DESC
        LIMIT {params.get('limit', 10)}
    """)
    
    return result

ListApi = ConsumptionApi[UserEvent, list_events](
    name="ListApi",
    source="UserEventTable"
)
```

### Advanced Endpoints
```python
class EventStats(BaseModel):
    event_type: str
    count: int
    unique_users: int
    first_seen: str
    last_seen: str

def get_event_stats(params: Dict[str, Any], utils) -> list[EventStats]:
    client = utils.client
    sql = utils.sql
    
    result = await client.query.execute(sql"""
        SELECT
            event_type,
            count() as count,
            count(distinct user_id) as unique_users,
            min(timestamp) as first_seen,
            max(timestamp) as last_seen
        FROM UserEventTable
        WHERE timestamp >= {params['start_date']}
        AND timestamp <= {params['end_date']}
        GROUP BY event_type
        ORDER BY count DESC
    """)
    
    return result

StatsApi = ConsumptionApi[EventStats, get_event_stats](
    name="StatsApi",
    source="UserEventTable"
)
```

## Security

### Authentication
```python
def get_secure_events(params: Dict[str, Any], utils) -> list[UserEvent]:
    # Check authentication
    if not utils.jwt:
        raise Exception("Authentication required")
    
    client = utils.client
    sql = utils.sql
    
    result = await client.query.execute(sql"""
        SELECT *
        FROM UserEventTable
        WHERE user_id = {params['user_id']}
    """)
    
    return result

SecureApi = ConsumptionApi[UserEvent, get_secure_events](
    name="SecureApi",
    source="UserEventTable",
    settings={
        "auth": {
            "required": True,
            "roles": ["admin", "analyst"]
        }
    }
)
```

### Rate Limiting
```python
def get_rate_limited_events(params: Dict[str, Any], utils) -> list[UserEvent]:
    client = utils.client
    sql = utils.sql
    
    result = await client.query.execute(sql"""
        SELECT *
        FROM UserEventTable
        LIMIT {params.get('limit', 10)}
    """)
    
    return result

RateLimitedApi = ConsumptionApi[UserEvent, get_rate_limited_events](
    name="RateLimitedApi",
    source="UserEventTable",
    settings={
        "rate_limit": {
            "requests": 100,
            "period": 60  # 100 requests per minute
        }
    }
)
```

## Best Practices

1. **API Design**
   - Use meaningful endpoint paths
   - Implement proper error handling
   - Document your APIs
   - Version your APIs

2. **Security**
   - Implement authentication
   - Use rate limiting
   - Validate input parameters
   - Sanitize SQL queries

3. **Performance**
   - Use appropriate caching
   - Optimize queries
   - Monitor API usage
   - Set reasonable limits

4. **Error Handling**
   - Return proper status codes
   - Provide error messages
   - Log errors
   - Monitor failures

## Example Usage

### Basic API
```python
def get_basic_events(params: Dict[str, Any], utils) -> list[UserEvent]:
    client = utils.client
    sql = utils.sql
    
    result = await client.query.execute(sql"""
        SELECT *
        FROM UserEventTable
        ORDER BY timestamp DESC
        LIMIT {params.get('limit', 10)}
    """)
    
    return result

BasicApi = ConsumptionApi[UserEvent, get_basic_events](
    name="BasicApi",
    source="UserEventTable"
)
```

### Advanced API
```python
def get_advanced_stats(params: Dict[str, Any], utils) -> list[EventStats]:
    client = utils.client
    sql = utils.sql
    
    result = await client.query.execute(sql"""
        SELECT
            event_type,
            count() as count,
            count(distinct user_id) as unique_users,
            min(timestamp) as first_seen,
            max(timestamp) as last_seen
        FROM UserEventTable
        WHERE timestamp >= {params['start_date']}
        AND timestamp <= {params['end_date']}
        GROUP BY event_type
        ORDER BY count DESC
    """)
    
    return result

AdvancedApi = ConsumptionApi[EventStats, get_advanced_stats](
    name="AdvancedApi",
    source="UserEventTable",
    settings={
        "auth": {
            "required": True,
            "roles": ["analyst"]
        },
        "rate_limit": {
            "requests": 100,
            "period": 60
        }
    }
)
```

## Choosing Between APIs

### Use EgressApi when:
- You need to surface table data directly
- You want simple filtering and pagination
- You don't need complex business logic
- You want type-safe parameter handling
- You need automatic parameter validation

### Use ConsumptionApi when:
- You need custom business logic
- You want to transform the data before returning
- You need to combine data from multiple sources
- You want fine-grained control over the response format
- You need to implement complex authentication logic

## Example: Converting from ConsumptionApi to EgressApi

### Before (ConsumptionApi):
```python
from moose_lib import ConsumptionApi
from pydantic import BaseModel

class BrainData(BaseModel):
    id: str
    timestamp: float
    value: float

def get_brain_data(params: Dict[str, Any], utils) -> list[BrainData]:
    client = utils.client
    sql = utils.sql
    
    result = await client.query.execute(sql"""
        SELECT * FROM brain_data
        WHERE timestamp >= {params.get('start_time')}
        AND timestamp <= {params.get('end_time')}
        ORDER BY timestamp DESC
        LIMIT {params.get('limit', 100)}
    """)
    
    return result

BrainDataApi = ConsumptionApi[BrainDataQuery, BrainData](
    source="brain_data"
)
```

### After (EgressApi):
```python
from moose_lib import EgressApi

get_brain_data = EgressApi(
    name="get_brain_data",
    table="brain_data",
    sql="""
        SELECT * FROM brain_data
        WHERE ({start_time:Nullable(Float64)} IS NULL OR timestamp >= {start_time:Nullable(Float64)})
          AND ({end_time:Nullable(Float64)} IS NULL OR timestamp <= {end_time:Nullable(Float64)})
        ORDER BY timestamp DESC
        LIMIT {limit:Int32}
    """,
    query_params={
        "limit": {"type": "int", "default": 100},
        "start_time": {"type": "float", "required": False},
        "end_time": {"type": "float", "required": False},
    }
)
```

## Troubleshooting

### Common Issues

1. **Parameter Type Errors**
   - Problem: Parameters not matching expected types
   - Solution: Use correct type annotations in query_params

2. **SQL Errors**
   - Problem: Invalid SQL syntax or missing columns
   - Solution: Test SQL queries directly in your database

3. **Performance Issues**
   - Problem: Slow queries or large result sets
   - Solution: Add proper indexes and use LIMIT

4. **Authentication Errors**
   - Problem: Missing or invalid authentication
   - Solution: Check API settings and authentication configuration

### SQL Query Syntax

When writing SQL queries in your query functions, follow these rules:

1. **Query String Format**
   ```python
   # CORRECT: Use triple quotes for multi-line queries
   result = client.query.execute("""
       SELECT * FROM brain_data
       WHERE id = {params.id}
       LIMIT 1
   """)

   # CORRECT: Use single quotes for simple queries
   result = client.query.execute(
       "SELECT * FROM brain_data LIMIT 1"
   )

   # INCORRECT: Do not use sql""" syntax
   result = client.query.execute(sql"""  # This will cause a syntax error
       SELECT * FROM brain_data
   """)
   ```

2. **Parameter Interpolation**
   ```python
   # CORRECT: Use params model attributes
   result = client.query.execute("""
       SELECT * FROM brain_data
       WHERE id = {params.id}
       AND timestamp >= {params.start_time}
       LIMIT {params.limit}
   """)

   # CORRECT: Use get() for optional parameters
   result = client.query.execute("""
       SELECT * FROM brain_data
       WHERE id = {params.id}
       LIMIT {params.get('limit', 100)}
   """)

   # INCORRECT: Do not use f-strings
   result = client.query.execute(f"""  # This is unsafe
       SELECT * FROM brain_data
       WHERE id = {params.id}
   """)
   ```

3. **Query Structure**
   ```python
   # CORRECT: Well-structured query with proper formatting
   result = client.query.execute("""
       SELECT 
           id,
           timestamp,
           value
       FROM brain_data
       WHERE id = {params.id}
       ORDER BY timestamp DESC
       LIMIT {params.limit}
   """)

   # CORRECT: Simple query for single record
   result = client.query.execute("""
       SELECT * FROM brain_data
       WHERE id = {params.id}
       LIMIT 1
   """)
   ```

### Common Issues and Solutions

1. **SQL Syntax Errors**
   - Problem: Using incorrect SQL string syntax
   - Solution: Use triple quotes for multi-line queries
   ```python
   # Wrong
   result = client.query.execute(sql"""
       SELECT * FROM brain_data
   """)
   
   # Correct
   result = client.query.execute("""
       SELECT * FROM brain_data
   """)
   ```

2. **Parameter Interpolation**
   - Problem: Unsafe parameter interpolation
   - Solution: Use the params model attributes
   ```python
   # Wrong
   result = client.query.execute(f"""
       SELECT * FROM brain_data
       WHERE id = {params.id}
   """)
   
   # Correct
   result = client.query.execute("""
       SELECT * FROM brain_data
       WHERE id = {params.id}
   """)
   ```

3. **Query Structure**
   - Problem: Poorly formatted or unsafe queries
   - Solution: Use proper SQL formatting and parameterization
   ```python
   # Wrong
   result = client.query.execute("""
       SELECT * FROM brain_data WHERE id=" + params.id
   """)
   
   # Correct
   result = client.query.execute("""
       SELECT * FROM brain_data
       WHERE id = {params.id}
       LIMIT 1
   """)
   ```

## Response Models

When creating a ConsumptionApi, you need to define both the query parameters model and the response model. For list responses, you should create a wrapper model:

```python
from moose_lib import ConsumptionApi, EgressConfig
from pydantic import BaseModel
from typing import List, Optional

# Define your base data model
class BrainData(BaseModel):
    id: str
    timestamp: float
    value: float

# Define your list response model
class BrainDataListResponse(BaseModel):
    items: List[BrainData]
    total: int = 0

# Define your query parameters model
class BrainDataQuery(BaseModel):
    start_time: Optional[float] = None
    end_time: Optional[float] = None
    limit: Optional[int] = 100

# Define your query function
def get_brain_data(params: BrainDataQuery, utils) -> BrainDataListResponse:
    client = utils.client
    sql = utils.sql
    
    result = client.query.execute("""
        SELECT * FROM brain_data
        WHERE ({params.start_time} IS NULL OR timestamp >= {params.start_time})
        AND ({params.end_time} IS NULL OR timestamp <= {params.end_time})
        ORDER BY timestamp DESC
        LIMIT {params.limit}
    """)
    
    if not result:
        return BrainDataListResponse(items=[])
    
    return BrainDataListResponse(
        items=[BrainData(**item) for item in result],
        total=len(result)
    )

# CORRECT: Use proper response model and EgressConfig
brain_data_api = ConsumptionApi[BrainDataQuery, BrainDataListResponse](
    "get_brain_data",
    query_function=get_brain_data,
    source="brain_data",
    config=EgressConfig()  # Empty config is valid
)

# INCORRECT: Using base model for list response
brain_data_api = ConsumptionApi[BrainDataQuery, BrainData](  # Wrong: should use list response model
    "get_brain_data",
    query_function=get_brain_data,
    source="brain_data",
    config=EgressConfig()
)

# INCORRECT: Invalid EgressConfig parameters
brain_data_api = ConsumptionApi[BrainDataQuery, BrainDataListResponse](
    "get_brain_data",
    query_function=get_brain_data,
    source="brain_data",
    config=EgressConfig(
        auth={"required": True}  # Wrong: invalid config parameter
    )
)
```

### Response Model Structure

1. **Base Data Model**
   - Define your core data structure
   - Use proper Python types
   - Match your database schema
   ```python
   class BrainData(BaseModel):
       id: str
       timestamp: float
       value: float
   ```

2. **List Response Model**
   - Wrap your base model in a list response
   - Include total count if needed
   - Use proper type hints
   ```python
   class BrainDataListResponse(BaseModel):
       items: List[BrainData]
       total: int = 0
   ```

3. **Query Function Return Type**
   - Must match the response model
   - Handle empty results properly
   - Convert database results to models
   ```python
   def get_brain_data(params: BrainDataQuery, utils) -> BrainDataListResponse:
       # ... query execution ...
       if not result:
           return BrainDataListResponse(items=[])
       return BrainDataListResponse(
           items=[BrainData(**item) for item in result],
           total=len(result)
       )
   ```

### EgressConfig Usage

1. **Basic Configuration**
   - Empty config is valid
   - Use only supported parameters
   - Follow configuration guidelines
   ```python
   # CORRECT
   config=EgressConfig()  # Empty config is valid

   # INCORRECT
   config=EgressConfig(
       auth={"required": True}  # Wrong: invalid parameter
   )
   ```

2. **Supported Parameters**
   - Check documentation for supported options
   - Use proper parameter types
   - Follow configuration patterns
   ```python
   # CORRECT
   config=EgressConfig(
       # Add supported parameters here
   )

   # INCORRECT
   config=EgressConfig(
       unsupported_param=True  # Wrong: unsupported parameter
   )
   ```

### Common Issues and Solutions

1. **Incorrect Response Model**
   - Problem: Using base model for list response
   - Solution: Create proper list response model
   ```python
   # Wrong
   def get_brain_data(params: BrainDataQuery, utils) -> List[BrainData]:
       # ...
   
   # Correct
   def get_brain_data(params: BrainDataQuery, utils) -> BrainDataListResponse:
       # ...
   ```

2. **Invalid EgressConfig**
   - Problem: Using unsupported parameters
   - Solution: Use only supported parameters
   ```python
   # Wrong
   config=EgressConfig(
       auth={"required": True}  # Wrong: invalid parameter
   )
   
   # Correct
   config=EgressConfig()  # Empty config is valid
   ```

3. **Empty Results**
   - Problem: Not handling empty results properly
   - Solution: Return empty list response
   ```python
   # Wrong
   if not result:
       return None  # Wrong: should return empty list response
   
   # Correct
   if not result:
       return BrainDataListResponse(items=[])
   ```

### Best Practices

1. **Response Models**
   - Create proper list response models
   - Include total count when needed
   - Use proper type hints

2. **Configuration**
   - Use empty EgressConfig if no settings needed
   - Follow configuration guidelines
   - Use only supported parameters

3. **Query Functions**
   - Return proper response model
   - Handle empty results
   - Convert database results to models

4. **Type Safety**
   - Use proper type hints
   - Match function signatures
   - Validate response structure