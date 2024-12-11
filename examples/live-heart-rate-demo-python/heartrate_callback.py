""" 
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.

Copyright (C) 2023 Fabrizio Smeraldi <fabrizio@smeraldi.net>
"""

""" Heart rate acquisition using a callback function """

import sys
import asyncio
from bleak import BleakScanner, BleakClient
import json
# Allow importing bleakheart from parent directory
sys.path.append('../')
from bleakheart import HeartRate
import requests
# Due to asyncio limitations on Windows, one cannot use loop.add_reader
# to handle keyboard input; we use threading instead. See
# https://docs.python.org/3.11/library/asyncio-platforms.html
if sys.platform=="win32":
    from threading import Thread
    add_reader_support=False
else:
    add_reader_support=True

# change these two parameters and see what happens.
# INSTANT_RATE is unsupported when UNPACK is False
UNPACK = True
INSTANT_RATE= UNPACK and False

def post_to_moose(data):
    url = "http://localhost:4000/ingest/BluetoothHRMSensorPacket"
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
        if device.name and "oura" in device.name.lower():
            return device
    return None


async def run_ble_client(device, hr_callback):
    """ This task connects to the sensor, starts heart rate notification
    and monitors connection and stdio for disconnects/user input. """

    def keyboard_handler(loop=None):
        """ Called by the asyncio loop when the user hits Enter, 
        or run in a separate thread (if no add_reader support). In 
        this case, the event loop is passed as an argument """
        input() # clear input buffer
        print (f"Quitting on user command")
        # causes the client task to exit
        if loop==None:
            quitclient.set() # we are in the event loop thread
        else:
            # we are in a separate thread - call set in the event loop thread
            loop.call_soon_threadsafe(quitclient.set)

    def disconnected_callback(client):
        """ Called by BleakClient if the sensor disconnects """
        print("Sensor disconnected")
        # signal exit
        quitclient.set() # causes the client task to exit

    # we use this event to signal the end of the client task
    quitclient=asyncio.Event()
    # the context manager will handle connection/disconnection for us
    async with BleakClient(device, disconnected_callback=
                           disconnected_callback) as client:
        print(f"Connected: {client.is_connected}")
        loop=asyncio.get_running_loop()
        if add_reader_support:
            # Set the loop to call keyboard_handler when one line of input is
            # ready on stdin
            loop.add_reader(sys.stdin, keyboard_handler)
        else:
            # run keyboard_handler in a daemon thread
            Thread(target=keyboard_handler, kwargs={'loop': loop},
                   daemon=True).start()
        print(">>> Hit Enter to exit <<<")
        # create the heart rate object; set callback and other
        # parameters
        heartrate=HeartRate(client, callback=hr_callback,
                            instant_rate=INSTANT_RATE,
                            unpack=UNPACK)
        # start notifications; bleakheart will start sending data to
        # the callback
        await heartrate.start_notify()
        # this task does not need to do anything else; wait until
        # user hits enter or the sensor disconnects
        await quitclient.wait()
        # no need to stop notifications if we are exiting the context
        # manager anyway, as they will disconnect the client; however,
        # it's easy to stop them if we want to
        if client.is_connected:
            await heartrate.stop_notify()
        if add_reader_support:
            loop.remove_reader(sys.stdin)


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
    # post_to_moose(payload)
    print(payload)

        
async def main():
    print("Scanning for BLE devices")
    device=await scan()
    if device==None:
        print("Polar device not found. If you have another compatible")
        print("device, edit the scan() function accordingly.")
        sys.exit(-4)
    print("After connecting, will print heart rate data in the form")
    if UNPACK:
        print("   ('HR', tstamp, (bpm, rr_interval), energy)")
    else:
        print("   ('HR', tstamp, (bpm, [rr1,rr2,...]), energy)")
    print("where tstamp is in ns, rr intervals are in ms, and")
    print("energy expenditure (if present) is in kJoule.")
    # client task will return when the user hits enter or the
    # sensor disconnects
    await run_ble_client(device, heartrate_callback)
    print("Bye.")


# execute the main coroutine
asyncio.run(main())
