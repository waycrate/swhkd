import glob
import libevdev
import os
from . log import LOG_UTILS

class INPUT_UTILS:
    def __init__(self):
        self.log_util=LOG_UTILS()

    async def check_keyboard(self,device_path) -> bool :
        with open(device_path, 'rb') as fd:
            device = libevdev.Device(fd)
        if device.has(libevdev.EV_KEY.KEY_ENTER):
            await self.log_util.log_info("Device {} is a keyboard".format(device_path))
            return True
        else:
            await self.log_util.log_error("Device {} is not a keyboard".format(device_path))
            return False

    async def get_keyboard_devices(self):
        devices = glob.glob('/dev/input/event*')
        keyboards = []
        for device in devices:
            out = await self.check_keyboard(device)
            if out ==True:
                keyboards.append(device)
        return keyboards

    async def run_system_command(self,command:str) -> None:
        os.system(f"setsid -f {command} 1>/dev/null 2>&1 3>&1")

    async def get_keyboard_events(self,device_path:str) -> None:
        with open(device_path, 'rb') as fd:
            device = libevdev.Device(fd)
            for event in device.events():
                if event.matches(libevdev.EV_MSC):
                    continue
                if event.matches(libevdev.EV_SYN.SYN_REPORT):
                    continue
                await self.log_util.log_info(event)
