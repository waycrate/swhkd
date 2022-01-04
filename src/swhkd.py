#!/usr/bin/python3
import asyncio
import getpass
import grp
import logging
import os
import pwd
import signal
import sys
from json import JSONDecodeError
from pathlib import Path

from utils.config import ConfigParser
from utils.input import SWHKDHelper
from utils.log import create_logger

logger = create_logger(name=__name__, level=logging.DEBUG)


class SWHKD:
    def __init__(self):
        signal.signal(signal.SIGINT, SWHKD.signal_handler)
        signal.signal(signal.SIGTERM, SWHKD.signal_handler)
        self.input_util = SWHKDHelper()
        self.user = getpass.getuser()
        self.config_parser = ConfigParser()

    @staticmethod
    def signal_handler(sig, frame):
        logger.warning("Quitting SWHKD")
        sys.exit(0)

    async def run_swhkd(self):
        # if os.getuid() == 0:
            # logger.critical("Refusing to run SWHKD as root")
            # sys.exit(1)

        # Permission check
        # groups = [g.gr_name for g in grp.getgrall() if self.user in g.gr_mem]
        # gid = pwd.getpwnam(self.user).pw_gid
        # groups.append(grp.getgrgid(gid).gr_name)
        # if "input" not in groups:
            # logger.error("User is in not in input group, exiting.")
            # sys.exit(1)

        # Config parsing
        config_paths = [
            Path("~/.config/swhkd/config.json")
        ]

        # Expand all the paths, (like convert ~ to /home/(username))
        config_paths = [path.expanduser().resolve() for path in config_paths]

        # Add XDG_CONFIG_HOME/swhkd/config.json if XDG_CONFIG_HOME is an
        # environment variable
        xdg_config_home = os.environ.get("XDG_CONFIG_HOME")
        if xdg_config_home is not None:
            config_home_path = Path(xdg_config_home) / "swhkd" / "config.json"
            logger.debug(f"XDG_CONFIG_HOME: {xdg_config_home}")
            logger.debug(f"New possible configuration path: "
                         f"{config_home_path}")
            config_paths.insert(0, config_home_path)
        else:
            logger.warning("XDG_CONFIG_HOME environment variable not set")

        # Try every configuration file location
        config = None
        for path in config_paths:
            try:
                config = await self.config_parser.parse(path)
            except FileNotFoundError:
                logger.debug(f"No file found at {path}")
            except JSONDecodeError:
                logger.warning(f"Invalid JSON file found at {path}")
            else:
                logger.info(f"Found configuration at {path}!")
        if config is None:
            logger.critical(f"No configuration files found! "
                            f"(checked {len(config_paths)} location(s))")
            # sys.exit(1)

        # Fetch events
        keyboards = await self.input_util.get_keyboard_devices()
        if len(keyboards) == 0:
            logger.critical("No keyboard devices found!")
            sys.exit(1)

        for keyboard in keyboards:
            await self.input_util.get_keyboard_events(keyboard)


if __name__ == "__main__":
    asyncio.run(SWHKD().run_swhkd())
