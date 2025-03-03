
# Add your models & start the development server to import these types
from app.datamodels.BluetoothHRMSensorPacket import BluetoothHRMSensorPacket
from app.datamodels.UNIFIED_HRM_MODEL import UNIFIED_HRM_MODEL
from moose_lib import StreamingFunction, Logger
from typing import Optional
from datetime import datetime, timezone
from pathlib import Path
import json

# Load the mock user db and return a dictionary of bluetooth devices
def load_bluetooth_devices():
    json_path = Path(__file__).parents[3] / 'mock-user-db.json'
    logger = Logger(action="BT")
    logger.info(f'Starting streaming function for Bluetooth HRM devices and loading mock user db from {json_path}')
    
    with open(json_path) as f:
        devices = json.load(f)
        
    bluetooth_devices = {
        int(device_id): device_info 
        for device_id, device_info in devices.items()
        if device_info.get('live_bt_device') == "True"
    }
    
    logger.info(f"Bluetooth device dict: {bluetooth_devices}")
    return bluetooth_devices

bluetooth_device_dict = load_bluetooth_devices()

def fn(source: BluetoothHRMSensorPacket) -> Optional[UNIFIED_HRM_MODEL]:
    user_name = bluetooth_device_dict[source.device_id]['user_name']
    user_id = bluetooth_device_dict[source.device_id]['user_id']
    timestamp_seconds = source.timestamp_ns / 1e9
    return UNIFIED_HRM_MODEL(
        user_id=user_id,
        user_name=user_name,
        device_id=source.device_id,
        hr_timestamp_seconds=timestamp_seconds,
        hr_value=source.heart_rate,
        rr_interval_ms=source.rr_interval_ms,
        processed_timestamp=datetime.now(timezone.utc),
    )

my_function = StreamingFunction(
    run=fn
)
