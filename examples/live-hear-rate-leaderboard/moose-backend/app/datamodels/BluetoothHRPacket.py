
from moose_lib import Key
from pydantic import BaseModel

class BluetoothHRPacket(BaseModel):
    device_id: Key[int]
    timestamp_ns: float
    heart_rate: int
    rr_interval_ms: int 