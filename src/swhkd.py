#!/usr/bin/python3

import asyncio
import getpass
import grp
import pwd
import signal
import sys

from utils.log import LOG_UTILS
from utils.input import INPUT_UTILS


class SWHKD:
    """ 
    Main Class.
    """
    def __init__(self):
        signal.signal(signal.SIGINT, self.signalhandler)
        signal.signal(signal.SIGTERM, self.signalhandler)
        self.log_util = LOG_UTILS()
        self.input_util = INPUT_UTILS()
        self.user = getpass.getuser()

    def signalhandler(self,sig,frame):
        print('\033[1;31mEXIT: Quitting SWHKD.\033[0m')
        sys.exit(0)

    async def run_swhkd(self):
        groups = [g.gr_name for g in grp.getgrall() if self.user in g.gr_mem]
        gid = pwd.getpwnam(self.user).pw_gid
        groups.append(grp.getgrgid(gid).gr_name)
        for group in groups:
            if group.lower() == "input":
                await self.log_util.log_warn("User is in input group, proceeding.")
                break;

        keyboards = await self.input_util.get_keyboard_devices()
        for keyboard in keyboards:
            await self.input_util.get_keyboard_events(keyboard)

asyncio.run(SWHKD().run_swhkd())
