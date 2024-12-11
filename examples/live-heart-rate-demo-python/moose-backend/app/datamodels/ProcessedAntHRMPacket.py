from moose_lib import Key, moose_data_model
from dataclasses import dataclass

# A flattened and validated version of the ANT HRM packet
# Unnest the byte array in the ANT+ packet and check they are valid

@moose_data_model
@dataclass
class ProcessedAntHRMPacket:
    device_id: Key[int]
    previous_beat_time: float
    last_beat_time: float
    calculated_heart_rate: float
    heart_beat_count: int