from moose_lib import MaterializedView, MaterializedViewOptions, Aggregated, AggregateFunction, ClickHouseEngines
from pydantic import BaseModel
from datetime import datetime

"""
        CREATE TABLE IF NOT EXISTS heart_rate_summary
        (
            user_name String,
            rounded_up_time Float64,
            processed_timestamp DateTime,
            avg_hr_per_second AggregateFunction(avg, Float64)
        )
        ENGINE = AggregatingMergeTree()
        PRIMARY KEY user_name
        ORDER BY (user_name, rounded_up_time, processed_timestamp);
        CREATE MATERIALIZED VIEW IF NOT EXISTS heart_rate_view
        TO heart_rate_summary
        AS
        SELECT
            user_name,
            ceil(hr_timestamp_seconds) as rounded_up_time,
            processed_timestamp,
            avgState(hr_value) as avg_hr_per_second
        FROM
            UNIFIED_HRM_MODEL_0_0
        GROUP BY user_name, rounded_up_time, processed_timestamp
        ORDER BY user_name, rounded_up_time;
        
"""

# Target table for the materialized view
class AggregateHeartRateSummaryPerSecond(BaseModel):
    user_name: str
    rounded_up_time: int
    processed_timestamp: datetime
    # Params: Aggregated [return_type_of_the_aggregated_function, AggregateFunction(agg_func="avg", param_types=[float])]
    avg_hr_per_second: Aggregated[float, AggregateFunction(agg_func="avg", param_types=[float])]


SELECT_QUERY = """
        SELECT
            user_name,
            ceil(hr_timestamp_seconds) as rounded_up_time,
            processed_timestamp,
            avgState(hr_value) as avg_hr_per_second
        FROM
            UNIFIED_HR_MODEL
        GROUP BY user_name, rounded_up_time, processed_timestamp
"""

aggregateHeartRateSummaryPerSecondMV = MaterializedView[AggregateHeartRateSummaryPerSecond](MaterializedViewOptions(
    select_statement=SELECT_QUERY,
    table_name="heart_rate_summary",
    materialized_view_name="heart_rate_view",
    order_by_fields=["user_name", "rounded_up_time"],
    engine=ClickHouseEngines.AggregatingMergeTree,
))  