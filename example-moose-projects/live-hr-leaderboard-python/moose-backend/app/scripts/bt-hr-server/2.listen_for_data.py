from moose_lib import task, Logger
from bleak import BleakClient
import requests
import json
from app.helpers.bleakheart import HeartRate
from asyncio import Event
import asyncio


INSTANT_RATE = False
UNPACK = True

num_payloads_passed = 0


@task(retries=3)
async def listen_for_data(data:dict):  # The name of your script
    """
    Connects to the Polar device and starts heart rate monitoring
    """
    logger = Logger(action="BT Listen")
    device_address = data["data"]["device_address"]
    quitclient = Event()
    reconnect_attempts = 0
    MAX_RECONNECT_ATTEMPTS = 3

    def disconnected_callback(client, logger):
        logger.warning("Device disconnected")
        quitclient.set()

    def post_to_moose(data, logger):
        """Helper function to post data to Moose endpoint"""
        global num_payloads_passed
        num_payloads_passed += 1
        url = "http://localhost:4000/ingest/BluetoothHRMSensorPacket"
        headers = {'Content-Type': 'application/json'}
        try:
            response = requests.post(url, headers=headers, data=json.dumps(data))
            if response.status_code != 200:
                logger.error(f"Error posting to Moose: {response.status_code} {response.text}")
                raise Exception(f"Failed to post to Moose: {response.status_code}")
        except Exception as e:
            logger.error(f"Error posting to Moose: {str(e)}")
            raise  # Let Temporal handle retry

    def heart_rate_callback(hr_data, logger):
        data_type, timestamp_ns, heart_rate_info, energy = hr_data
        bpm, rr_interval = heart_rate_info
        
        payload = {
            "device_id": data["data"]["device_address"],
            "timestamp_ns": timestamp_ns,
            "heart_rate": bpm,
            "rr_interval_ms": rr_interval,
        }
        post_to_moose(payload, logger)
        logger.info(f"Heart rate data: {payload}")

    async with BleakClient(device_address, disconnected_callback=lambda client: disconnected_callback(client, logger)) as client:    
        logger.info(f"Connected to device: {data['data']['device_name']}")
        
        heartrate = HeartRate(
            client, 
            callback=lambda hr_data: heart_rate_callback(hr_data, logger),
            instant_rate=INSTANT_RATE,
            unpack=UNPACK
        )
        
        await heartrate.start_notify()
        await quitclient.wait()
        
        if client.is_connected:
            await heartrate.stop_notify()

    logger = Logger(action="BT HR Listen")
    logger.info(f"Listening for data from device: {data}")
    logger.info(f"Data: {data['data']['device_address']}")

    # The return value is the output of the script.
    # The return value should be a dictionary with at least:
    # - step: the step name (e.g., "extract", "transform")
    # - data: the actual data being passed to the next step
    return {
        "step": "listen_for_data",  # The step name is the name of the script
        "data": {
            "device_address": device_address,
            "num_payloads_passed": num_payloads_passed
        }     # The data being passed to the next step (4MB limit)
    }