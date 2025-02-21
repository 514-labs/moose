from moose_lib import task
import requests
import json
import random
import time
import math

@task()
def send_data_to_moose(data: dict):
    """
    This script mocks N users who are wearing an ANT+ heart rate monitor.
    It sends data to Moose four times per second indefinitely
    This simulates a realistic ANT+ sensor with byte rollover
    For reference view the pdf in this repo under /docs/ANT-HRM-Packet-Format.pdf
    """
    # Define the API endpoint URL
    url = "http://localhost:4000/ingest/RawAntHRMPacket"
    headers = {'Content-Type': 'application/json'}

    # Update for the number of devices
    device_ids = [12345, 12346, 12347, 12348, 12349]
    
    start_time = time.time()

    # Initialize device-specific data
    device_data = {
        device_id: {
            'hr_history': [],
            'previous_beat_time': 0,
            'last_beat_time': 0,
            'beat_count': 0,
            'time_rollover': 0,
            'packet_number': 0,
            'rhr': random.randint(55, 85),
            'hr_max': random.randint(200, 220),
            'hr_event': False,
            'phase': random.random(),
        } for device_id in device_ids
    }

    # TODO: Add sig term error processing

    while True:
        for device_id in device_ids:
            time_elapsed = time.time() - start_time
            
            if device_data[device_id]['hr_event'] or device_data[device_id]['packet_number'] == 0:
                device_data[device_id]['target_hr'] = generate_realistic_heart_rate(
                    time_elapsed,
                    base_hr=device_data[device_id]['rhr'],
                    max_hr=device_data[device_id]['hr_max'], 
                    phase=device_data[device_id]['phase']
                )
                device_data[device_id]['hr_event'] = False
            
            ant_packet = generate_ant_hrm_packet(device_id, device_data[device_id])
            json_data = json.dumps(ant_packet)
            
            try:
                response = requests.post(url, data=json_data, headers=headers)
                if response.status_code == 200:
                    print(f"Successfully sent packet: {ant_packet}")
                else:
                    print(f"Failed to send packet: {response.status_code}, {response.text}")
            except requests.exceptions.RequestException as e:
                print(f"An error occurred: {e}")

        time.sleep(0.25)

    # Note: This return statement will never be reached due to the infinite loop
    return {
        "step": "send_data_to_moose",
        "data": None
    }

def generate_ant_hrm_packet(device_id: int, device_data: dict):
    """
    Generates a synthetic ANT+ HRM packet based on a target heart rate.
    Returns the packet, a boolean indicating if a heart rate event occurred,
    and the updated last_beat_time and beat_count.
    """
    transmission_type = 1
    device_type = 120  # ANT+ device type for HRM
    page_number = 0x04  # Page 4 per ANT+ HRM specification (https://err.no/tmp/ANT_Device_Profile_Heart_Rate_Monitor.pdf)

    # Absolute time scale in seconds
    current_time = device_data['packet_number'] * 0.25 * 1024  # Convert to 1/1024 second resolution
    next_beat_time = (60 * 1024 / device_data['target_hr']) + device_data['last_beat_time'] + (device_data['time_rollover'] * 65536)


    if device_data['last_beat_time'] > 65536:
        device_data['time_rollover'] += 1
        device_data['last_beat_time'] = device_data['last_beat_time'] % 65536

    # Boolean indicating if a heart beat event occurred
    heart_rate_event = current_time > next_beat_time

    if heart_rate_event:
        device_data['previous_beat_time'] = device_data['last_beat_time'] % 65536
        device_data['last_beat_time'] = next_beat_time % 65536
        device_data['beat_count'] = (device_data['beat_count'] + 1) % 255 
        device_data['hr_event'] = True
        device_data['hr_history'].append(device_data['target_hr'])


    # Assemble the 8-byte data payload
    data_payload = [
        page_number,  # Byte 0: Page Number
        0xFF,  # Byte 1: Manufacturer Specific
        int(device_data['previous_beat_time']) & 0xFF,  # Byte 2: Previous Heart Beat Event Time LSB
        (int(device_data['previous_beat_time']) >> 8) & 0xFF,  # Byte 3: Previous Heart Beat Event Time MSB
        int(device_data['last_beat_time']) & 0xFF,  # Byte 4: Heart Beat Event Time LSB
        (int(device_data['last_beat_time']) >> 8) & 0xFF,  # Byte 5: Heart Beat Event Time MSB
        device_data['beat_count'],  # Byte 6: Heart Beat Count
        device_data['hr_history'][-1] if len(device_data['hr_history']) > 0 else 0  # Byte 7: Computed Heart Rate (last value in history)
    ]

    # Construct the ANT+ packet as a dictionary
    ant_packet = {
        "device_id": device_id,
        "packet_count": device_data['packet_number'],
        "ant_hrm_packet": data_payload
    }

    device_data['packet_number'] += 1
    return ant_packet


def generate_realistic_heart_rate(time_elapsed, base_hr=75, max_hr=180, session_duration=300, last_hr=None, phase=0.5):
    # Calculate the phase of the workout (0 to 1)
    phase = time_elapsed / session_duration
    
    # Simplified heart rate curve calculation
    hr_curve = 0.5 * (1 + math.tanh(10 * (phase - 0.5)))
    
    # Calculate target heart rate
    target_hr = base_hr + (max_hr - base_hr) * hr_curve 
    
    # Add random variation
    variation = random.uniform(-3, 5)
    new_hr = target_hr + variation
    
    # Apply smoothing if there's a previous heart rate
    if last_hr is not None:
        new_hr = 0.7 * last_hr + 0.3 * new_hr
    
    # Ensure the heart rate stays within realistic bounds
    final_hr = int(max(base_hr, min(round(new_hr), max_hr)))
    
    return final_hr