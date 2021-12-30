#!/usr/bin/python3
import json
from . log import LOG_UTILS

class CONFIG_PARSER:
    def __init__(self):
        self.LOG_UTILS = LOG_UTILS()

    async def parse(self,config_file):
        output=[]
        with open(config_file, 'r') as file:
            contents = json.loads(file.read())
        for arr in contents:
            pair=[]
            for key, value in arr.items():
                pair.append(value)
            output.append(pair)
        return output
