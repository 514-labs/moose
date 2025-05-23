from app.datamodels.AppleWatchHRPacket import AppleWatchHRPacket
from app.datamodels.UnifiedHRPacket import UnifiedHRPacket
from typing import Optional
from datetime import datetime, timezone
from pathlib import Path
import json
from moose_lib import Logger, Key

def load_device_dict():
    json_path = Path(__file__).parents[2] / 'mock-user-db.json'
    logger = Logger(action="SF")
    # logger.info(f'Starting streaming function and loading mock user db from {json_path}')

    with open(json_path) as f:
        device_dict = json.load(f)
    # Filter to only include devices marked as live bluetooth devices
    device_dict = {k: v for k, v in device_dict.items() if v.get('live_bt_device') == "True"}
    
    return device_dict

all_bluetooth_device_dict = load_device_dict()

logger = Logger(action="SF")
# logger.info(f'all_bluetooth_device_dict: {all_bluetooth_device_dict}')

# Capture the moment this module is first imported as the streaming start time (UTC)
stream_start_time = datetime.now(timezone.utc)
# logger.info(f"Apple Watch streaming function start time: {stream_start_time.isoformat()}")

def apple_watch_to_unified(source: AppleWatchHRPacket) -> UnifiedHRPacket:
    device_id = source.device_id  # This is a Key[int]
    device_id_str = str(device_id)
    device_dict = all_bluetooth_device_dict[device_id_str]
    user_name = device_dict.get('user_name')
    user_id = device_dict.get('user_id')  # This should already be an integer

    # Time elapsed (in seconds) since the streaming function/module was first loaded
    elapsed_seconds = (datetime.now(timezone.utc) - stream_start_time).total_seconds()

    return UnifiedHRPacket(
        user_id=user_id,  # UnifiedHRPacket will handle the Key[int] conversion
        user_name=user_name,
        device_id=int(device_id_str),  # Convert to plain int
        hr_timestamp_seconds=elapsed_seconds,
        hr_value=source.heart_rate_data,
        rr_interval_ms=0, # not included in apple watch
        processed_timestamp=datetime.now(timezone.utc),
    )
