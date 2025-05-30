#!/usr/bin/env python3
# Requirements: bleak, requests
import asyncio
from bleak import BleakScanner, BleakClient
import requests
from moose_lib import task, Logger

HEART_RATE_CHAR_UUID = "00002a37-0000-1000-8000-00805f9b34fb"
INGEST_URL = "http://localhost:4000/ingest/apple_health_watch_packet"

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
    devices = await BleakScanner.discover()
    if not devices:
        logger.error("No BLE devices found.")
        return

    device = None
    for d in devices:
        if d.name and "iphone" in d.name.lower():
            device = d
            break

    if not device:
        logger.error("No iPhone found. Available devices:")
        for d in devices:
            logger.info(f"  {d.name} ({d.address})")
        return

    logger.info(f"Connecting to: {device.name} ({device.address})")

    async with BleakClient(device) as client:
        logger.info(f"Connected: {client.is_connected}")

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