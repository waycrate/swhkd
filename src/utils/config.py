#!/usr/bin/python3
import json
import logging

from .log import create_logger
from pathlib import Path

logger = create_logger(name=__name__, level=logging.DEBUG)


class ConfigParser:
    """
    Does the configuration parsing.
    """

    def __init__(self):
        pass

    async def parse(self, config_file: Path) -> list[list[str, ...]]:
        """
        Parses the configuration file.

        :param config_file: The path to the config file.
        :return: A list of lists of strings.
        """
        logger.debug(f"Loading configuration file from {config_file}")
        contents = json.loads(config_file.read_text())
        output = []
        for arr in contents:
            pair = []
            for key, value in arr.items():
                pair.append(value)
            output.append(pair)
        return output
