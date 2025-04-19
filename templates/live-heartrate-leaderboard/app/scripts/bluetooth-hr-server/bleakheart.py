""" 
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.

Copyright (C) Fabrizio Smeraldi - fabrizio@smeraldi.net, 2023
"""

__version__ = "0.1.2"

import asyncio as aio
from time import time_ns
from collections import defaultdict
from bleak import BleakGATTCharacteristic, BleakClient
from inspect import iscoroutinefunction


class HeartRate:
    """ Access heart rate service as specified by the BLE SIG - this
    should work with all devices following the specification. Frames 
    are written to an asyncio queue or passed to a callback. Heart rate
    can be as returned by the device (typically averaged) or the instant
    rate, if RR intervals are supported. Heart rate data can be passed to
    the queue/callback as received (a frame may contain data for more than
    one heartbeat) or unpacked as individual heartbeats.

    If unpacking is not required, data is pushed to the queue/passed to 
    the callback in the following format:
        ('HR', tstamp, (avghr, rrlist), energy)
    where 'HR' is a constant string, tstamp is the (client) time stamp in ns, 
    avghr the average heart rate as detected by the sensor, rrlist is a list
    of RR values in the frame (in ms, if supported), and energy is the energy 
    expenditure (in kJoule, if supported).

    If unpacking is required, individual rr interval information is 
    separated and data is formatted as follows:
        ('HR', t_est, (hr, rr), energy)
    where t_est is the estimated time stamp of the individual heartbeat, and
    hr can be either the average heart rate returned by the sensor or the 
    instant heart rate as computed from the specific RR interval.
    """
    
    CHARACTERISTIC="00002a37-0000-1000-8000-00805f9b34fb"

    def __init__(self, client: BleakClient, queue: aio.Queue=None,
                 callback=None, contact_callback=None,
                 contact_lost_callback=None,
                 instant_rate=False, unpack=True):
        """
        Init the HeartRate object.

        Args:

        client: the BleakClient connection object for the BLE device
        queue:  an asyncio queue onto which heart rate data is pushed;
                alternatively, you can specify a callback
        callback: a function to which heart rate data is passed. If queue
                is specified, this parameter is ignored.
        contact_callback: a function/coroutine that is called or awaited
                when the sensor skin contact bit goes from 0 to 1 (if
                supported). Also see the good_contact attribute
        contact_lost_callback: a function/coroutine that is called or 
                awaited when the sensor skin contact bit goes from 1 to 0
                (if supported). Also see the lost_contact attribute
        instant_rate: if True, the average heart rate returned by the sensor
                is replaced by the instant rate computed from individual
                RR intervals. Only works if unpack it True
        unpack: if True, data in sensor frames is  unpacked and processed as 
                individual heartbeats. Only works if RR intervals are 
                supported

        Attributes:

        contact_detection: True if the sensor supports skin contact detection,
                False otherwise. This attribute is None until the first
                frame is received.
        good_contact: an awaitable asyncio Event that is set when the sensor
                reports good skin contact (if supported) and cleared when it
                reports poor contact.
        lost_contact: an awaitable asyncio Event that is set when the sensor
                reports poor skin contact (if supported) and cleared when it
                reports good contact.
        """
        if unpack==False and instant_rate==True:
            raise RuntimeError("instant_rate only supported when unpack==True")
        self.client=client
        self.instant_rate=instant_rate
        self.unpack=unpack
        # must have callback or queue for hr signal. callback ignoed
        # if queue is specified
        if queue==None and callback==None:
            raise RuntimeError("No queue or callback given for HR signal")
        # _callback is passed sensor frames by handler
        self._callback=queue.put_nowait if queue!=None else callback
        self._callback_is_coro=iscoroutinefunction(self._callback)
        # contact detection
        self.good_contact=aio.Event()
        self.lost_contact=aio.Event()
        self.contact_detection=None
        self.filter_nocontact=False
        # contact established callback
        self._contact_callback=contact_callback
        self._contact_callback_is_coro=(iscoroutinefunction(contact_callback)
                                        if contact_callback!=None else None)
        # contact lost callback
        self._lost_callback=contact_lost_callback
        self._lost_callback_is_coro=(iscoroutinefunction(self._lost_callback)
                                     if self._lost_callback!=None else None)


    def _decode(self, data: bytearray):
        """
        See www.bluetooth.com/specifications/specs/heart-rate-service-1-0/ 
        for the structure of the frame.
        NOTE: Polar H10 does not support contact bit or energy expenditure,
        so these features are untested
        """
        # the first byte contains flags
        flags = data[0]
        payload={}

        # format for hr: 1 or 2 bits, little endian
        uint8_format = (flags & 1) == 0  # bit 0, can change
        energy_expenditure = (flags & 8)>0 # bit 3, can change
        rr_intervals = (flags & 16) >0  # bit 4, can change
        
        self.contact_detection= (flags & 4) > 0 # bit 2, static
        if self.contact_detection:
            # good contact if bit is set
            payload['contact']= (flags & 2) >0 # bit 1, can change
            
        if uint8_format:
            payload['hr'] = data[1]
            offset=2
        else:
            payload['hr'] = int.from_bytes(data[1:3], 'little', signed=False)
            offset=3
        if energy_expenditure:
            nrg = int.from_bytes(data[offset:offset+2], 'little', signed=False)
            offset += 2
            payload['nrg']=nrg

        if rr_intervals:
            payload['rr']=[]
            for i in range(offset, len(data), 2):
                rr = int.from_bytes(data[i:i+2],
                                    'little', signed=False)
                # Polar H7, H9, and H10 record RR intervals
                # in 1024-th parts of a second. Convert this
                # to milliseconds.
                rr = round(rr * 1000 / 1024)
                payload['rr'].append(rr)
        return payload

    
    async def _handler(self, characteristic: BleakGATTCharacteristic,
                       data: bytearray):
        """ Callback handler for notifications """
        tstamp=time_ns()
        payload=self._decode(data)
        # contact detection supported
        if self.contact_detection:
            if payload['contact'] and not self.good_contact.is_set():
                self.lost_contact.clear()
                self.good_contact.set()
                if self._contact_callback!=None:
                    if self._contact_callback_is_coro:
                        await self._contact_callback()
                    else:
                        self._contact_callback()
            if not payload['contact'] and not self.lost_contact.is_set():
                self.good_contact.clear()
                self.lost_contact.set()
                if self._lost_callback!=None:
                    if self._lost_callback_is_coro:
                        await self._lost_callback()
                    else:
                        self._lost_callback()
            if not payload['contact'] and self.filter_nocontact:
                return
        avghr=payload['hr']
        rrlist=payload.get('rr', [])
        energy=payload.get('ee', None)

        if not self.unpack:
            if self._callback_is_coro:
                await self._callback(('HR', tstamp, (avghr, rrlist), energy))
            else:
                self._callback(('HR', tstamp, (avghr, rrlist), energy))
        else:
            # unpack each individual heartbeat
            if len(rrlist)==0:
                return
            t_est=tstamp-sum(rrlist)*1000000 # nanoseconds
            for rr in rrlist:
                t_est+=rr * 1000000 # nanoseconds
                hr=round(60000.0/rr) if self.instant_rate else avghr
                if self._callback_is_coro:
                    await self._callback(('HR', t_est, (hr, rr), energy))
                else:
                    self._callback(('HR', t_est, (hr, rr), energy))


    async def start_notify(self, filter_nocontact=False):
        """ Start heart rate notifications; this will start writing
        data to the queue or passing it to the callback as it is
        received from the sensor. If filter_nocontact is True, 
        frames received without the 'contact' bit set are discarded 
        (the contact bit must be supported by the sensor)
        """
        await self.client.start_notify(HeartRate.CHARACTERISTIC,
                                        self._handler)
        self.good_contact.clear()
        self.lost_contact.clear()
        self.filter_nocontact=filter_nocontact

    async def stop_notify(self):
        """ Stop heart rate notifications """
        await self.client.stop_notify(HeartRate.CHARACTERISTIC)
        self.good_contact.clear()
        self.lost_contact.clear()
