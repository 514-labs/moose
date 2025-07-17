
# This file is where you can define your API templates for consuming your data

from moose_lib import MooseClient, ConsumptionApi, MooseCache
from pydantic import BaseModel, Field
from typing import Optional
from app.views.bar_aggregated import barAggregatedMV

# Query params are defined as Pydantic models and are validated automatically
class QueryParams(BaseModel):
    order_by: Optional[str] = Field(
        default="total_rows",
        pattern=r"^(total_rows|rows_with_text|max_text_length|total_text_length)$",
        description="Must be one of: total_rows, rows_with_text, max_text_length, total_text_length"
    )
    limit: Optional[int] = Field(
        default=5,
        gt=0,
        le=100,
        description="Must be between 1 and 100"
    )
    start_day: Optional[int] = Field(
        default=1,
        gt=0,
        le=31,
        description="Must be between 1 and 31"
    )
    end_day: Optional[int] = Field(
        default=31,
        gt=0,
        le=31,
        description="Must be between 1 and 31"
    )

class QueryResult(BaseModel):
    day_of_month: int
    total_rows: int
    rows_with_text: int
    max_text_length: int
    total_text_length: int
    
    
## The run function is where you can define your API logic
def run(client: MooseClient, params: QueryParams):

    # Create a cache
    cache = MooseCache()
    cache_key = f"bar:{params.order_by}:{params.limit}:{params.start_day}:{params.end_day}"

    # Check for cached query results
    cached_result = cache.get(cache_key)
    if cached_result and len(cached_result) > 0:
        return cached_result

    start_day = params.start_day
    end_day = params.end_day
    limit = params.limit
    order_by = params.order_by
    
    query = f"""
    SELECT 
        day_of_month,
        total_rows,
        rows_with_text,
        max_text_length,
        total_text_length
    FROM {barAggregatedMV.target_table.name} 
    WHERE day_of_month >= {start_day} 
    AND day_of_month <= {end_day} 
    ORDER BY {order_by} DESC
    LIMIT {limit}
    """    

    result = client.query.execute(query, {"order_by": order_by, "start_day": start_day, "end_day": end_day, "limit": limit})

    # Cache query results
    cache.set(result, cache_key, 3600)  # Cache for 1 hour

    return result


bar = ConsumptionApi[QueryParams, QueryResult](name="bar", query_function=run)