#!/usr/bin/env python3
# Requirements: bleak, requests
import asyncio
from bleak import BleakScanner, BleakClient
import requests
from moose_lib import task, Logger

HEART_RATE_CHAR_UUID = "00002a37-0000-1000-8000-00805f9b34fb"
INGEST_URL = "http://localhost:4000/ingest/apple_health_watch_packet"

BLE_DEVICE_NAME = "iPhone"
DEBUG_BLE_MODE = False

def parse_heart_rate(data):
    # Standard BLE Heart Rate Measurement characteristic parsing
    # See: https://www.bluetooth.com/specifications/specs/heart-rate-profile-1-0/
    flag = data[0]
    hr_format = flag & 0x01
    if hr_format == 0:
        hr_value = data[1]
    else:
        hr_value = int.from_bytes(data[1:3], byteorder='little')
    return hr_value

@task
async def apple_watch_hr_client():
    logger = Logger("apple_watch_hr_client")
    logger.info("Scanning for Apple Watch devices...")
    devices_dict = await BleakScanner.discover(return_adv=True)
    if not devices_dict:
        logger.error("No BLE devices found.")
        return {
            "task": "apple_watch_hr_client",
            "data": {
                "device_id": None,
                "error": "No iPhone found"
            }
        }

    device = None
    for address, (ble_device, adv_data) in devices_dict.items():
        logger.info(f"Device name: {ble_device.name} ({address})")
        if ble_device.name and BLE_DEVICE_NAME.lower().strip() in ble_device.name.lower().strip():
            logger.info(f"Found {BLE_DEVICE_NAME}: {ble_device.name} ({ble_device.address})")
            device = ble_device

    if not device:
        logger.error(f"No {BLE_DEVICE_NAME} found from BleakScanner")
        return {
            "task": "apple_watch_hr_client",
            "data": {
                "device_id": None,
                "error": "No iPhone found"
            }
        }

    logger.info(f"Connecting to: {device.name} ({device.address})")

    async with BleakClient(device) as client:
        logger.info(f"Connected: {client.is_connected}")
        # Get all services
        if DEBUG_BLE_MODE:
            services = await client.get_services()
            logger.info("Services:")
            for service in services:
                logger.info(f"Service: {service.uuid}")
                for char in service.characteristics:
                    logger.info(f"  Characteristic: {char.uuid}")
                    logger.info(f"    Properties: {char.properties}")
                    logger.info(f"    Handle: {char.handle}")
                    if "read" in char.properties:
                        try:
                            value = await client.read_gatt_char(char.uuid)
                            logger.info(f"    Value: {value}")
                        except Exception as e:
                            logger.info(f"    Could not read value: {e}")

        def hr_callback(sender, data):
            try:
                heart_rate = parse_heart_rate(data)
                payload = {
                    "device_id": device.address,
                    "heart_rate_data": heart_rate
                }
                response = requests.post(INGEST_URL, json=payload)
                if response.status_code == 200:
                    logger.info(f"Sent: {payload}")
                else:
                    logger.error(f"Failed to send data: {response.status_code} {response.text}")
            except Exception as e:
                logger.error(f"Error in callback: {e}")

        logger.info(f"Subscribing to heart rate notifications on {HEART_RATE_CHAR_UUID}...")
        try:
            await client.start_notify(HEART_RATE_CHAR_UUID, hr_callback)
        except Exception as e:
            logger.error(f"Error subscribing to heart rate notifications: {e}")
            return
        logger.info("Subscribed! Waiting for data...")
        try:
            while True:
                await asyncio.sleep(1)
        except KeyboardInterrupt:
            logger.info("Disconnecting...")
        await client.stop_notify(HEART_RATE_CHAR_UUID)
    logger.info("Disconnected.")
   
    return {
        "task": "apple_watch_hr_client",
        "data": {
            "device_id": device.address
        }
    }