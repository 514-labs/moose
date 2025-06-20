from pydantic import BaseModel
from moose_lib import Key
from typing import List

class AppleWatchHRPacket(BaseModel):
    device_id: Key[str]
    heart_rate_data: int