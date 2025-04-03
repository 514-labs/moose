
# This file is where you can define your API templates for consuming your data

from moose_lib import MooseClient
from moose_lib.dmv2 import ConsumptionApi
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
    
    start_day = params.start_day
    end_day = params.end_day
    limit = params.limit
    order_by = params.order_by
    
    query = f"""
    SELECT 
        day_of_month,
        {order_by}
    FROM {barAggregatedMV.target_table.name} 
    WHERE day_of_month >= {start_day} 
    AND day_of_month <= {end_day} 
    ORDER BY {order_by} DESC
    LIMIT {limit}
    """    
   
    return client.query.execute(query, {"order_by": order_by, "start_day": start_day, "end_day": end_day, "limit": limit})

