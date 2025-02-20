from moose_lib import StreamingFunction
from datetime import datetime
from app.datamodels.models import UserActivity, ParsedActivity

def parse_activity(activity: UserActivity) -> ParsedActivity:
    return ParsedActivity(
        eventId=activity.eventId,
        timestamp=datetime.fromisoformat(activity.timestamp),
        userId=activity.userId,
        activity=activity.activity,
    )

my_function = StreamingFunction(
    run=parse_activity
)
