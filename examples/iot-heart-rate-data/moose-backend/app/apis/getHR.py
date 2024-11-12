# This file is where you can define your API templates for consuming your data
# All query_params are passed in as strings, and are used within the sql tag to parameterize you queries
from datetime import datetime, timedelta, timezone
from moose_lib import MooseClient, cli_log, CliLogData

def run(client: MooseClient, params):
    input_timestamp = params.get("timestamp", [datetime.now(timezone.utc).isoformat()])
    cli_log(CliLogData(message=f'input_timestamp: {input_timestamp}'))
    # Convert the string timestamp to a datetime object
    dt = datetime.fromisoformat(input_timestamp[0].replace('Z', '+00:00'))
    
    # Round down to the nearest second
    rounded_dt = dt.replace(microsecond=0)
    rounded_dt = rounded_dt - timedelta(seconds=1)
    # Format the rounded timestamp
    formatted_timestamp = rounded_dt.strftime("%Y-%m-%dT%H:%M:%S")
        
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
