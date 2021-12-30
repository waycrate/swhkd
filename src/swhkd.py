#!/usr/bin/python3
import asyncio
import getpass
import grp
import os
import pwd
import signal
import sys

from utils.log import LOG_UTILS
from utils.input import INPUT_UTILS
from utils.config import CONFIG_PARSER


class SWHKD:
    def __init__(self):
        signal.signal(signal.SIGINT, self.signalhandler)
        signal.signal(signal.SIGTERM, self.signalhandler)
        self.log_util = LOG_UTILS()
        self.input_util = INPUT_UTILS()
        self.user = getpass.getuser()
        self.config_parser = CONFIG_PARSER()

    def signalhandler(self,sig,frame):
        print('\033[1;31mEXIT: Quitting SWHKD.\033[0m')
        sys.exit(0)

    async def run_swhkd(self):
        if os.getuid() == 0:
            await self.log_util.log_error('Refusing to run SWHKD as root.')
            sys.exit(1)

        # Permission check
        groups = [g.gr_name for g in grp.getgrall() if self.user in g.gr_mem]
        gid = pwd.getpwnam(self.user).pw_gid
        groups.append(grp.getgrgid(gid).gr_name)
        if "input" not in groups:
                await self.log_util.log_error("User is in not in input group, exiting.")
                sys.exit(1)

        # Config parsing
        try:
            config = await self.config_parser.parse("{0}swhkd/config.json".format(os.environ.get("XDG_CONFIG_HOME")))
        except FileNotFoundError:
            try:
                config = await self.config_parser.parse("~/.config/swhkd/config.json")
            except FileNotFoundError:
                await self.log_util.log_error("Failed to parse config files.")
                sys.exit(1)

        # Fetch events
        keyboards = await self.input_util.get_keyboard_devices()
        if not keyboards:
            await self.log_util.log_error("No keyboard devices found.")
            sys.exit(1)
        for keyboard in keyboards:
            await self.input_util.get_keyboard_events(keyboard)

if __name__ == "__main__":
    asyncio.run(SWHKD().run_swhkd())
