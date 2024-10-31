from datetime import datetime, timezone, timedelta
from moose_lib import MooseClient, cli_log, CliLogData

# AI imports 
import instructor
from pydantic import BaseModel, Field
from openai import OpenAI 
  
import os
import json
from json import JSONEncoder

class CoachingOutput(BaseModel):
    coaching_message: str = Field(description="A short message that is encouraging and motivating but also data-driven based on the query")
    analysis_message: str = Field(description="A short message that is data-driven based on the query")
# Patch the Instructor client with the OpenAI client 
# Locally running LLM - can be replaced with OpenAI/Anthropic/Gemini etc.
instructor_client = instructor.from_openai(
    OpenAI(
        base_url="http://localhost:11434/v1",
        api_key="ollama",  # required, but unused
    ),
    mode=instructor.Mode.JSON,
)

def run(client: MooseClient, params):
    # Get the analysis window from the params - default to 5 seconds
    # analysis_window = int(params.get("analysis_window", [5])[0])
    # timestamp = datetime.now(timezone.utc) - timedelta(seconds=analysis_window)
    # Round down to the nearest second
    # rounded_dt = timestamp.replace(microsecond=0)
    # # Format the rounded timestamp
    # formatted_timestamp = rounded_dt.strftime("%Y-%m-%dT%H:%M:%S")

    formatted_timestamp = '2024-10-28 15:53:48'
    sql_response = client.query(
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
    if not sql_response: 
        return {
            "coaching_message": "",
            "analysis_message": ""
        }
    # Extract structured data from natural language
    cli_log(CliLogData(message=f'sql_response: {sql_response}'))

    # Play around with the prompt with Joj to get the best results
    ai_response = instructor_client.chat.completions.create(
        model="llama3.2",
        response_model=CoachingOutput,
        messages=[
            {"role": "system", "content": """
             You are a live coach helping a team of indoor cyclist stay motivated and have fun. 
             Your job is to analyze the data and provide a short message which will be displayed on a live leaderboard.
             You will be given a list of users and their average heart rate for a second over the last 5 seconds.,
             Give them short messages that are encouraging and motivating but also data-driven based on this query:
             {query}
             """},
            {"role": "user", "content": str(sql_response)}
        ],
    )

    return {
        "coaching_message": ai_response.coaching_message,
        "analysis_message": ai_response.analysis_message
    }


