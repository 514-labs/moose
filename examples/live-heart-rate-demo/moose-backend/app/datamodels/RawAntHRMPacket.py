from moose_lib import Key, moose_data_model
from dataclasses import dataclass
from datetime import datetime


@moose_data_model
@dataclass
class RawAntHRMPacket:
    device_id: Key[int]
    packet_count: int
    ant_hrm_packet: list[int]
