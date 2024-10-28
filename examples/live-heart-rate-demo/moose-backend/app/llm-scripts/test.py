import instructor
from pydantic import BaseModel, Field
from openai import OpenAI


# Patch the OpenAI client
client = instructor.from_openai(OpenAI())
# Define your desired output structure
from pydantic import BaseModel, Field

class CoachingOutput(BaseModel):
    coaching_message: str = Field(description="A short message that is encouraging and motivating but also data-driven based on the query")
    analysis_message: str = Field(description="A short message that is data-driven based on the query")

ai_response = client.chat.completions.create(
    model="gpt-4o",
    response_model=CoachingOutput,
    messages=[
        {"role": "system", "content": """
            You are a live coach helping a team of indoor cyclist stay motivated and have fun. 
            Your job is to analyze the data and provide a short message which will be displayed on a live leaderboard.
            You will be given a list of users and their average heart rate for a second over the last 5 seconds.,
            Give them short messages that are encouraging and motivating but also data-driven based on this query:
            {query}
            """},
        {"role": "user", "content": """
         here's a table of hr data: [
            Joj: 87, Chris: 102, Arman: 94, Olivia: 105, Tim: 100,
            Joj: 89, Chris: 100, Arman: 93, Olivia: 101, Tim: 150
         ]
         """}
    ],
)

print({
    "coaching_message": ai_response.coaching_message,
    "analysis_message": ai_response.analysis_message
}) 
