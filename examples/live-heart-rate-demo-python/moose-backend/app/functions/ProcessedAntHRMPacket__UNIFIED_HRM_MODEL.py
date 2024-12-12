
# Add your models & start the development server to import these types
from app.datamodels.ProcessedAntHRMPacket import ProcessedAntHRMPacket
from app.datamodels.UNIFIED_HRM_MODEL import UNIFIED_HRM_MODEL
from moose_lib import StreamingFunction
from typing import Optional
from datetime import datetime, timezone
 
# TODO: Turn this into a supabase table and query
device_dict = {
    12345: {
        'user_id': 0,
        'user_name': 'Joj',
    },
    12345: {
        'user_id': 1,
        'user_name': 'Olivia',
    },
    12346: {
        'user_id': 2,
        'user_name': 'Tim',
    },
    12347: {
        'user_id': 3,
        'user_name': 'Alex',
    },
    12348: {
        'user_id': 4,
        'user_name': 'Arman',
    },
    12349: {
        'user_id': 5,
        'user_name': 'Samia',
    }
}

def fn(source: ProcessedAntHRMPacket) -> Optional[UNIFIED_HRM_MODEL]:
    user_id = device_dict[source.device_id]['user_id']
    user_name = device_dict[source.device_id]['user_name']

    return UNIFIED_HRM_MODEL(
        user_id=user_id,
        user_name=user_name,
        device_id=source.device_id,
        hr_timestamp_seconds=source.last_beat_time,
        hr_value=source.calculated_heart_rate,
        rr_interval_ms=(source.last_beat_time - source.previous_beat_time) * 1000,
        # Time in UTC when data is processed
        processed_timestamp=datetime.now(timezone.utc),
    )

my_function = StreamingFunction(
    run=fn
)
