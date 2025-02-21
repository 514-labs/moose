from moose_lib import Blocks
from moose_lib import Blocks

"""
ClickHouse block for computing per-second heart rate averages.

This models a realistic scenario where a device sends HR data at a rate of 4Hz
By averaging across the second, users get smoother and more accurate representation of their HR

This block sets up a data pipeline that continuously aggregates heart rate data into
per-second averages for each user. It consists of:

1. A summary table (heart_rate_summary) that stores aggregated heart rate data using
   ClickHouse's AggregatingMergeTree engine for efficient incremental aggregation.

2. A materialized view (heart_rate_view) that automatically processes incoming heart
   rate data from UNIFIED_HRM_MODEL_0_0, computing running averages per second.

Note: To query final average values, use avgMerge() on the avg_hr_per_second column.
"""

block = Blocks(
    setup=[
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
        """,
        """
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
    ],
    teardown=[
        # Array of SQL statements for tearing down resources
        "DROP VIEW IF EXISTS heart_rate_view",
        "DROP TABLE IF EXISTS heart_rate_summary",
    ],
)