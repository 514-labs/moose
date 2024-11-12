from moose_lib import Blocks
from moose_lib import Blocks

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