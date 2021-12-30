#!/usr/bin/python3
import asyncio
import getpass
import grp
import os
import pwd
import signal
import sys
from pathlib import Path

from utils.config import CONFIG_PARSER
from utils.input import INPUT_UTILS
from utils.log import LOG_UTILS


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
        paths = []
        if os.environ.get("XDG_CONFIG_HOME") is not None:
            paths.append(Path(os.environ.get("XDG_CONFIG_HOME")) / "swhkd/config.json")
        paths.append(Path("~/.config/swhkd/config.json"))
        paths.append(Path.cwd() / "config.json")
        paths.append(Path.cwd().parent / "config.json")

        config = None
        # Try every config and break on first success
        for path in paths:
            try:
                config = await self.config_parser.parse(path)
            except FileNotFoundError:
                continue
            else:
                break

        # No config file found
        if config is None:
            await self.log_util.log_error("No valid configuration file found.")
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
