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


class BatteryLevel:
    """ Access battery characteristic; the read method returns battery 
    charge status as a percentage """
    CHARACTERISTIC="00002a19-0000-1000-8000-00805f9b34fb"

    def __init__(self, client: BleakClient):
        """ Init the BatteryLevel object. 
        Args: client - the BleakClient connection object for the BLE device
        """
        self.client=client

    async def read(self):
        """ Reads the batter level characteristic. Returns the battery level
        as a percentage of full charge """
        data=await self.client.read_gatt_char(BatteryLevel.CHARACTERISTIC)
        return data[0]



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

        
class PolarMeasurementData:
    """ Access measurements provided through the Polar Measurement Data
    interface: Electrocardiogram, Acceleration, Photoplethysmography, 
    Peak-to-peal Interval, Gyroscope, Magnetometer.  Data is pushed to an 
    asynchronous queue or passed to a callback as requested, in the format 
    (DTYPE, tstamp, payload). Here DTYPE is a string from measurement_types, 
    tstamp is the sensor time stamp in ns, and payload is the requested 
    measurement data. The time stamp is converted to standard epoch time 
    using a single offset computed at the time the first data frame is 
    received. ECG and acceleration data (as supported by Polar H10) are 
    decoded; other measurement data are streamed, but they are returned 
    as raw bytearray.

    Acceleration samples are returned as tuples of values along the three
    axes (x,y,z), in milliG; ECG is returned as a list of integer samples in 
    microVolt (on the H10, ECG  sampling frequency is 130Hz and encoding 
    is 14 bit). In either case, the time stamp refers to the last sample 
    of the list constituting the payload.
    """
    # BLE characteristics
    PMDCTRLPOINT="FB005C81-02E7-F387-1CAD-8ACD2D8DF0C8"
    PMDDATAMTU="FB005C82-02E7-F387-1CAD-8ACD2D8DF0C8"
    
    # electrocardiogram, photoplethysmography, acceleration,
    # peak to peak interval, gyroscope, magnetometer.
    measurement_types=['ECG', 'PPG', 'ACC', 'PPI', 'rfu', 'GYRO', 'MAG']
    # 'rfu' = reserved for future use. Use list.index() to get the code
    # for each string.
    op_codes={'GET':0x01, 'START': 0x02, 'STOP': 0x03}
    settings=['SAMPLE_RATE', 'RESOLUTION', 'RANGE', 'rfu', 'CHANNELS']
    # Choice of default settings for measurements supported by Polar H10.
    # Sampling rate is in Hz, Resolution in bit, Range in multiples of G.
    default_settings={'ECG': {'SAMPLE_RATE': 130, 'RESOLUTION': 14},
                      'ACC': {'SAMPLE_RATE': 200, 'RESOLUTION': 16,
                              'RANGE': 2 }}
    # these are Polar sensor errors; bleakheart errors will use negative
    # error codes
    error_msgs=['SUCCESS', 'INVALID OP CODE', 'INVALID MEASUREMENT TYPE',
                'NOT SUPPORTED', 'INVALID LENGTH', 'INVALID PARAMETER',
                'ALREADY IN STATE', 'INVALID RESOLUTION',
                'INVALID SAMPLE RATE', 'INVALID RANGE',
                'INVALID MTU', 'INVALID NUMBER OF CHANNELS',
                'INVALID STATE', 'DEVICE IN CHARGER']

    def __init__(self, client: BleakClient,
                 ecg_queue:aio.Queue=None, acc_queue:aio.Queue=None,
                 raw_queue:aio.Queue=None, callback=None):
        """" Init the PolarMeasurementData object.

        Args:

        client:    the BleakClient connection object for the BLE device
        ecg_queue: an asyncio queue onto which decoded ECG data is pushed;
                   if not specified, data will be sent to the callback
        acc_queue: an asyncio queue onto which decoded acceleration data is 
                   pushed; if unspecified, data will be passed to callback
        raw_queue: an asyncio queue onto which all other measurement data is 
                   pushed in raw format; if not specified, data will be 
                   passed to the callback
        callback:  a function or coroutine function to which all measurement
                   data not pushed onto a queue is passed. 
        """
        self.client=client
        self.ecg_queue=ecg_queue
        self.acc_queue=acc_queue
        self.raw_queue=raw_queue
        if callback==None:
            callback=self._no_callback
        self._ecg_callback=ecg_queue.put_nowait if ecg_queue!=None else callback
        self._acc_callback=acc_queue.put_nowait if acc_queue!=None else callback
        self._raw_callback=raw_queue.put_nowait if raw_queue!=None else callback
        self._ecg_callback_is_coro=iscoroutinefunction(self._ecg_callback)
        self._acc_callback_is_coro=iscoroutinefunction(self._acc_callback)
        self._raw_callback_is_coro=iscoroutinefunction(self._raw_callback)
        self._ctrl_lock=aio.Lock()
        self._ctrl_recv=aio.Event() # ctrl response ready
        self._ctrl_response=None
        self._notifications_started=False
        self._time_offset=None

    def _no_callback(self, payload):
        """ Used to raise an error if no queue or callback has been 
        specified for the type of frame """
        raise RuntimeError(f"No callback for {payload[0]} frames "+
                           "(try passing queue or callback)")

    async def _start_notifications(self):
        """ Start notifications from the control point and data
        characteristic """
        # handle responses from control point
        await self.client.start_notify(self.PMDCTRLPOINT,
                                       self._pmd_ctrl_handler)
        # handle responses form data point
        await self.client.start_notify(self.PMDDATAMTU,
                                       self._pmd_data_handler)
        self._notifications_started=True


    async def _pmd_ctrl_handler(self, characteristic: BleakGATTCharacteristic,
                                data: bytearray):
        """ Handler for control point responses. Stores the response in 
        self._ctrl_response and notifies the method that sent the control 
        query """
        if data[0]!=0xF0:
            raise RuntimeError(f"Invalid response from PMD control point")
        self._ctrl_response=data
        # method that sent the ctlr query is awaiting this 
        self._ctrl_recv.set()
        
    async def _pmd_ctrl_request(self, request: bytearray):
        """ Sends a control request to the PMD control point. Awaits the
        response with a timeout of 10 seconds. """
        # time to start notifications if not done yet
        if not self._notifications_started:
            await self._start_notifications()
        # make sure no other request can be made to ctrl point until
        # it  has responded. 
        async with self._ctrl_lock:
            self._ctrl_recv.clear()
            await self.client.write_gatt_char(self.PMDCTRLPOINT, request)
            # wait for response handler to set the event
            # can hang if response is lost, hence the timeout
            # async with aio.timeout(10): is not available in Python ver 3.8
            # using wait_for on instead
            await aio.wait_for(self._ctrl_recv.wait(), timeout=10)
            # grab response before releasing lock
            response=self._ctrl_response
        return response
        
    async def _pmd_data_handler(self, characteristic: BleakGATTCharacteristic,
                                data: bytearray):
        """ Receives responses from the PMD data point and delegates to
        the appropriate function for decoding. Pushes results to the 
        relevant queue or calls/awaits the callback. Data are formatted
        as tuples: (MEASR, timestamp, payload) where MEASR is a string
        identifying the measurement, followed by the sensor timestamp and 
        the list of samples (for measurements other than ECG or ACC, the 
        raw dataframe is returned as the payload).
        """
        meas=self.measurement_types[data[0]]
        timestamp=int.from_bytes(data[1:9], 'little', signed=False)
        frametype=data[9]
        try:
            timestamp+=self._time_offset
        except TypeError:
            self._time_offset=time_ns()-timestamp
            timestamp+=self._time_offset
        
        if meas=='ECG':
            payload=self._decode_ecg_data(data)
            if self._ecg_callback_is_coro:
                await self._ecg_callback(('ECG', timestamp, payload))
            else:
                self._ecg_callback(('ECG', timestamp, payload))
        elif (meas=='ACC') and (frametype==1):
            payload=self._decode_acc_data(data)
            if self._acc_callback_is_coro:
                await self._acc_callback(('ACC', timestamp, payload))
            else:
                self._acc_callback(('ACC', timestamp, payload))
        else:
            # send raw data to queue or callback
            if self._raw_callback_is_coro:
                await self._raw_callback((meas, timestamp, data))
            else:
                self._raw_callback((meas, timestamp, data))
        
    
    def _decode_ecg_data(self, data):
        """ Decodes ECG data frames from the device.

        Args: 
            data: the raw ECG frame from the device. This is an array of 
                  3-byte little-endian signed integers
        Returns:
            A list of ECG values in microvolt, as integers
        """
        if data[9]!=0x00:
            raise ValueError("Invalid ECG frame type")
        if (len(data)-10)%3!=0:
            raise ValueError("Bad ECG data frame length")
        microvolt=[]
        for offset in range(10, len(data), 3):
            muv=int.from_bytes(data[offset:offset+3],
                               'little',
                               signed=True)
            microvolt.append(muv)
        return microvolt

    def _decode_acc_data(self, data):
        """ Decode acceleration data frame type 0x01 (x,y,z, 16 bit signed 
        int, unis: mG); this is the type of frame returned by the H10 strap

        Args:
            data: the raw ACC frame from the device. Only frame type 0x01 is 
            supported
        Returns:
            A list of tuples of the form (x,y,z) where x, y, and z are 
            integers measuring the acceleration along the three axes
            in milliG.
        """
        if data[9]!=0x01:
            raise ValueError(f"Unsupported ACC frame type {data[9]:02x}")
        if (len(data)-10)%6!=0:
            raise ValueError("Bad ACC data frame length")
        milli_g=[]
        for offset in range(10, len(data), 6):
            x=int.from_bytes(data[offset  :offset+2], 'little', signed=True)
            y=int.from_bytes(data[offset+2:offset+4], 'little', signed=True)
            z=int.from_bytes(data[offset+4:offset+6], 'little', signed=True)
            milli_g.append((x,y,z))
        return milli_g
    

    async def available_measurements(self):
        """ Reads the PMD Control Point to obtain the available
        measurements. 

        Returns:
            A list of available measurements; for the H10 strap this should 
            be ['ECG', 'ACC'] 

        Raises:
            A RuntimeError is raised in case of invalid read response from
            the CTRL Point
        """
        data=await self.client.read_gatt_char(self.PMDCTRLPOINT)
        if data[0]!=0x0F:
            raise RuntimeError(f"Invalid read response from PMD control point")
        flags=data[1]
        measr=[ meas for i, meas in enumerate(self.measurement_types)
                if  flags & (1<<i) > 0 ]
        return measr
    
        
    async def available_settings(self, measurement):
        """ Requests allowed parameters for the given measurement from 
        the device.

        Args:
            measurement: one of the strings in self.measurement_types

        Returns:
            A dictionary with the available settings for the measurement
            as keys. For each key, the value is a list of allowed values
            for that setting. 

        Errors:
            A dictionary with an error code of -1 and an error message is
            returned if the CTRL Point response times out.

        Raises:
            A RuntimeError is raised if the CTRL Point response has more
            than one frame or an illegal length.
        """
        try:
            mtype=self.measurement_types.index(measurement)
        except ValueError as e:
            e.args=(f'Unknown measurement type: {measurement}',)
            raise e
        cmd=self.op_codes['GET']
        try:
            data=await self._pmd_ctrl_request(bytearray([cmd, mtype]))
        except aio.TimeoutError:
            return {'error_code': -1,
                    'error_msg': 'PMD CTLR Point response timeout'}
        if data[1]!=cmd or data[2]!=mtype:
            raise RuntimeError(f"Invalid response from PMD control point")
        params=defaultdict(list)
        params['error_code']=data[3]
        params['error_msg']=self.error_msgs[data[3]]
        if data[3]>0: # an error occurred
            return params
        if data[4]!=0x00:
            raise RuntimeError("Multiple frames in PMD ctrl response "
                               "not supported")
        offset=5
        try:
            while offset< len(data):
                # decode the response here and return the dictionary
                parname=self.settings[data[offset]]
                howmany=data[offset+1]
                offset+=2
                for i in range(howmany):
                    val=int.from_bytes(data[offset:offset+2],
                                       'little', signed=False)
                    params[parname].append(val)
                    offset+=2
        except IndexError:
            raise RuntimeError("PMD response has wrong length")
        return params

    async def start_streaming(self, measurement, **settings):
        """ Start streaming, check ctrl point response for errors.
        Return a tuple with the error code (0 for success), error
        message, and the raw response so that unsupported parameters
        can be handled by the caller. """
        try:
            meas_type=self.measurement_types.index(measurement)
        except ValueError:
            return  (-3, f"Unknown measurement type: {measurement}", None)

        # default settings if present
        if measurement in self.default_settings:
            params=self.default_settings[measurement].copy()
        else:
            params={}
        # allow giving settings in lowercase
        for s,v in settings.items():
            params[s.upper()]=v
        cmd=self.op_codes['START']
        req=bytearray([cmd, meas_type])
        for s,v in params.items():
            req.extend([self.settings.index(s),
                        0x01]) # array length
            req.extend(v.to_bytes(2, 'little', signed=False))
        try:
            response=await self._pmd_ctrl_request(req)
        except aio.TimeoutError:
            return (-1, 'PMD CTLR Point response timeout', None)
        if response[1]!=cmd or response[2]!=meas_type:
            return (-2, 'Invalid CTRL point response', None)
        err_code=response[3]
        err_msg=self.error_msgs[err_code]
        # Verity ACC reponse has FACTOR parameter, not handled here
        return (err_code, err_msg, response)


    async def stop_streaming(self, measurement):
        """ Stop streaming, check ctrl point response for errors.
        Return a tuple with the error code (0 for success) and error
        message. """
        try:
            meas_type=self.measurement_types.index(measurement)
        except ValueError:
            return  (-3, f"Unknown measurement type: {measurement}")

        cmd=self.op_codes['STOP']
        req=bytearray([cmd, meas_type])
        try:
            response=await self._pmd_ctrl_request(req)
        except aio.TimeoutError:
            return (-1, 'PMD CTLR Point response timeout')
        if response[1]!=cmd or response[2]!=meas_type:
            return (-2, 'Invalid CTRL point response')
        err_code=response[3]
        err_msg=self.error_msgs[err_code]
        return (err_code, err_msg)
