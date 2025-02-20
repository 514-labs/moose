
from moose_lib import Key, moose_data_model
from dataclasses import dataclass
from datetime import datetime

@moose_data_model 
@dataclass
class BluetoothHRMSensorPacket:
    device_id: Key[str]
    timestamp_ns: float
    heart_rate: int
    rr_interval_ms: int 