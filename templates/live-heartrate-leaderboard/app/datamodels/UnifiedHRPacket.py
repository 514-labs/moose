from moose_lib import Key
from pydantic import BaseModel
from datetime import datetime
from typing import Optional

class UnifiedHRPacket(BaseModel):
    user_id: Key[int]
    user_name: str
    device_id: int
    hr_timestamp_seconds: float
    hr_value: float
    rr_interval_ms: Optional[float]
    processed_timestamp: datetime
    # hr_max: float
    # hr_zone: int