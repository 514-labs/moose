# Add your models & start the development server to import these types
from app.datamodels.RawAntHRMPacket import RawAntHRMPacket
from app.datamodels.ProcessedAntHRMPacket import ProcessedAntHRMPacket
from moose_lib import StreamingFunction
from typing import Optional
from datetime import datetime

 
device_dict = {}
 
def fn(source: RawAntHRMPacket) -> Optional[ProcessedAntHRMPacket]:
    # Track the number of times the time byte has been incremented
    # 2 Bytes ==> 16 bits ==> 65536 values of 1/1024 seconds ==> 64 seconds
    if source.device_id not in device_dict:
        device_dict[source.device_id] = {
            'previous_beat_time_rollover': 0,
            'last_beat_time_rollover': 0,
            'heart_beat_rollover': 0
        }

    # Merges the 2 bytes based on the LSB and MSB
    # TODO: Add the rollover to the previous and last beat time
    previous_beat_time_seconds = (float((source.ant_hrm_packet[3] << 8) | source.ant_hrm_packet[2]) + (device_dict[source.device_id]['previous_beat_time_rollover'] * 65536)) / 1024
    last_beat_time_seconds = (float((source.ant_hrm_packet[5] << 8) | source.ant_hrm_packet[4]) + (device_dict[source.device_id]['last_beat_time_rollover'] * 65536)) / 1024
    calculated_heart_rate = source.ant_hrm_packet[7]

    # Invalid heart rate value per ANT+ HRM spec
    if calculated_heart_rate < 0 or calculated_heart_rate > 255:
        # TODO: oute to dead letter queue
        return None
    
    heart_beat_count = source.ant_hrm_packet[6] + (device_dict[source.device_id]['heart_beat_rollover'] * 256)
    return ProcessedAntHRMPacket(
        device_id=source.device_id,
        previous_beat_time=previous_beat_time_seconds,
        last_beat_time=last_beat_time_seconds,
        heart_beat_count = heart_beat_count,
        calculated_heart_rate=calculated_heart_rate, 
    ) 

my_function = StreamingFunction(
    run=fn
)
  