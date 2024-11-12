
from moose_lib import Key, moose_data_model
from dataclasses import dataclass
from datetime import datetime


# Custom business logic to handle multiple HRM protocols
# Enriched with data from OLTP database

@moose_data_model 
@dataclass
class UNIFIED_HRM_MODEL:
    user_id: Key[int]
    user_name: str
    device_id: int
    hr_timestamp_seconds: float
    hr_value: float
    rr_interval_ms: float
    processed_timestamp: datetime
    # hr_max: float
    # hr_zone: int