from pydantic import BaseModel
from moose_lib import Key
from typing import List

class AppleWatchHRPacket(BaseModel):
    device_id: Key[int]
    heart_rate_data: int
    # timestamp_ns: float
    # rr_interval_ms: int