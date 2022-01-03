import logging
import os
from pathlib import Path

import libevdev

from .log import create_logger

logger = create_logger(name=__name__, level=logging.DEBUG)


class SWHKDHelper:
    def __init__(self):
        pass

    @staticmethod
    async def check_keyboard(device_path: Path) -> bool:
        with open(device_path, "rb") as fd:
            device = libevdev.Device(fd)
        if device.has(libevdev.EV_KEY.KEY_ENTER):
            logger.info(f"Device {device_path} is a keyboard")
            return True
        else:
            logger.warning(f"Device {device_path} is not a keyboard")
            return False

    @staticmethod
    async def get_keyboard_devices():
        devices = Path("/dev/input/").glob("event*")
        keyboards = []
        for device in devices:
            if await SWHKDHelper.check_keyboard(device):
                keyboards.append(device)
        return keyboards

    @staticmethod
    async def run_system_command(command: str):
        os.system(f"setsid -f {command} 1>/dev/null 2>&1 3>&1")

    @staticmethod
    async def get_keyboard_events(device_path: Path):
        with device_path.open("rb") as fd:
            device = libevdev.Device(fd)
            for event in device.events():
                if event.matches(libevdev.EV_MSC):
                    continue
                elif event.matches(libevdev.EV_SYN.SYN_REPORT):
                    continue
                logger.debug(event)
