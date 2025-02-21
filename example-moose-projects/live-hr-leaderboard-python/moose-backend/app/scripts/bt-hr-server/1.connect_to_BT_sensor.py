from moose_lib import task, Logger
from bleak import BleakScanner
import asyncio
from app.helpers.bleakheart import HeartRate

@task(retries=3)
async def connect_to_BT_sensor(data):
    """Connects to a Polar device and returns the device name and address"""
    logger = Logger(action="BT-Workflow")
    logger.info("Scanning for Polar device")
    
        # Run the async scanner in a synchronous way
    devices = await BleakScanner.discover()

    for device in devices:
        logger.info(f"Found device: {device.name}")
        if device.name and "polar" in device.name.lower():
            logger.info(f"Found Polar device: {device.name}")
            return {
                "task": "scan_for_polar_device",
                "data": {
                    "device_name": device.name,
                    "device_address": device.address,
                    "status": "success"
                }
            }
        
    # Rely on temporal retries logic 
    raise Exception("No Polar device found")

