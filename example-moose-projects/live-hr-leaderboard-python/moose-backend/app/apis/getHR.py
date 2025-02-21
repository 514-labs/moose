
# This file is where you can define your API templates for consuming your data
# All query_params are passed in as strings, and are used within the sql tag to parameterize you queries

from moose_lib import MooseClient, Logger
from datetime import datetime, timezone, timedelta

def run(client: MooseClient, params):
    logger = Logger(action="API")
    input_timestamp = params.get("timestamp", [datetime.now(timezone.utc).isoformat()])
    dt = datetime.fromisoformat(input_timestamp[0].replace('Z', '+00:00'))
    rounded_dt = dt.replace(microsecond=0)
    rounded_dt = rounded_dt - timedelta(seconds=1)
    formatted_timestamp = rounded_dt.strftime("%Y-%m-%dT%H:%M:%S")
    logger.info(f"Fetching HR data for timestamp: {formatted_timestamp}")
    return client.query(
        '''
        SELECT
            user_name,
            avg_heart_rate,
            last_processed_timestamp
        FROM
        (
            SELECT
                user_name,
                max(processed_timestamp) AS last_processed_timestamp,
                rounded_up_time,
                avgMerge(avg_hr_per_second) AS avg_heart_rate
            FROM heart_rate_summary
            GROUP BY user_name, rounded_up_time
            ORDER BY user_name, rounded_up_time
        )
        WHERE last_processed_timestamp >= toDateTime({timestamp})
        ''',
        {"timestamp": formatted_timestamp}
    )
