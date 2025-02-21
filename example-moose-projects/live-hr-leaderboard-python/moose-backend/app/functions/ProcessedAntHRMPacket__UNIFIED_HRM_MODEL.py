
# Add your models & start the development server to import these types
from app.datamodels.ProcessedAntHRMPacket import ProcessedAntHRMPacket
from app.datamodels.UNIFIED_HRM_MODEL import UNIFIED_HRM_MODEL
from moose_lib import StreamingFunction, Logger
from typing import Optional
from datetime import datetime, timezone
from pathlib import Path
import json


json_path = Path(__file__).parents[3] / 'mock-user-db.json'
logger = Logger(action="SF")
logger.info(f'Starting streaming function and loading mock user db from {json_path}')

with open(json_path) as f:
    device_dict = json.load(f)

logger.info(f"Device dict: {device_dict}")
def fn(source: ProcessedAntHRMPacket) -> Optional[UNIFIED_HRM_MODEL]:
    device_id = str(source.device_id)
    user_id = device_dict[device_id]['user_id']
    user_name = device_dict[device_id]['user_name']

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
