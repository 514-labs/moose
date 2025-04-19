# Note this live bluetooth server is only tested on the Polar H10
# To connect other devices, edit scan() function string in the code below accordingly

import asyncio
from bleak import BleakScanner, BleakClient
import json
from bleakheart import HeartRate
import requests
from moose_lib import task, Logger


def post_to_moose(data):
    url = "http://localhost:4000/ingest/bluetooth_hr_packet"
    headers = {'Content-Type': 'application/json'}
    response = requests.post(url, headers=headers, data=json.dumps(data))
    if response.status_code != 200:
        print(f"Error posting to moose: {response.status_code} {response.text}")

async def scan():
    """ Scan for a Polar device. If you have another compatible device,
    edit the string in the code below accordingly """
    devices = await BleakScanner.discover()
    for device in devices:
        print(f"Found device: {device.name} - Address: {device.address} - device: {device.details}")
        if device.name and "polar" in device.name.lower():
            return device
    return None

async def run_ble_client(device, hr_callback):
    """ This task connects to the sensor and starts heart rate notification """
    def disconnected_callback(client):
        """ Called by BleakClient if the sensor disconnects """
        print("Sensor disconnected")
        # signal exit
        quitclient.set() # causes the client task to exit

    # we use this event to signal the end of the client task
    quitclient = asyncio.Event()
    
    # the context manager will handle connection/disconnection for us
    async with BleakClient(device, disconnected_callback=disconnected_callback) as client:
        print(f"Connected: {client.is_connected}")
        
        # create the heart rate object; set callback and other parameters
        heartrate = HeartRate(client, callback=hr_callback,
                            instant_rate=INSTANT_RATE,
                            unpack=UNPACK)
        
        # start notifications; bleakheart will start sending data to the callback
        await heartrate.start_notify()
        
        # wait until the sensor disconnects
        await quitclient.wait()
        
        # stop notifications if we're still connected
        if client.is_connected:
            await heartrate.stop_notify()

def heartrate_callback(data):
    """ This callback is sent the heart rate data and does all the 
    processing. You should ensure it returns before the next 
    frame is received from the sensor. 

    In this example, we simply print decoded heart rate data as it 
    is received """
    data_type, timestamp_ns, heart_rate_info, energy = data
    bpm, rr_interval = heart_rate_info

    payload = {
        # Hardcoded for now
        "device_id": 1111,
        "timestamp_ns": timestamp_ns,
        "heart_rate": bpm,
        "rr_interval_ms": rr_interval,
    }
    post_to_moose(payload)
    print(payload)

@task
async def generate_bluetooth_hr_data():
    logger = Logger(action="gen_bluetooth_hr")
    """
    This script connects to a Polar Bluetooth heart rate monitor and streams
    heart rate data to Moose. It will continue running until the device disconnects.
    """
    print("Scanning for BLE devices")
    device = await scan()
    if device == None:
        print("Polar device not found. If you have another compatible")
        print("device, edit the scan() function accordingly.")
        return {
            "task": "gen_bluetooth_hr",
            "data": {
                "status": "error",
                "message": "Polar device not found"
            }
        }

    await run_ble_client(device, heartrate_callback)
    
    return {
        "task": "gen_bluetooth_hr",
        "data": {
            "status": "completed",
            "device_name": device.name,
            "device_address": device.address
        }
    }
