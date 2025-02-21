
# Add your models & start the development server to import these types
from app.datamodels.BluetoothHRMSensorPacket import BluetoothHRMSensorPacket
from app.datamodels.UNIFIED_HRM_MODEL import UNIFIED_HRM_MODEL
from moose_lib import StreamingFunction
from typing import Optional
from datetime import datetime, timezone

bluetooth_device_dict = {
    1111: {
        'user_id': 6,
        'user_name': 'Chris',
    }
}

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
