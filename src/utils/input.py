#!/usr/bin/python3

import glob
import libevdev
from . log import LOG_UTILS

class INPUT_UTILS:
    def __init__(self):
        self.log_util=LOG_UTILS()

    async def check_keyboard(self,device_path) -> bool :
        """
        Check if the device is a keyboard.
        """
        fd = open(device_path, 'rb')
        device = libevdev.Device(fd)
        fd.close()
        if device.has(libevdev.EV_KEY.KEY_ENTER):
            await self.log_util.log_info("Device {} is a keyboard".format(device_path))
            return True
        else:
            await self.log_util.log_error("Device {} is not a keyboard".format(device_path))
            return False

    async def get_keyboard_devices(self):
        """
        Get all keyboard devices.
        """
        devices = glob.glob('/dev/input/event*')
        keyboards = []
        for device in devices:
            out = await self.check_keyboard(device)
            if out ==True:
                keyboards.append(device)
        return keyboards

    async def get_keyboard_events(self,device_path:str) -> None:
        """
        Get all device events.
        """
        with open(device_path, 'rb') as fd:
            device = libevdev.Device(fd)
            for event in device.events():
                if event.matches(libevdev.EV_MSC):
                    continue
                print(event)
