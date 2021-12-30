#!/usr/bin/python3
import json
from . log import LOG_UTILS

class CONFIG_PARSER:
    def __init__(self):
        self.LOG_UTILS = LOG_UTILS()

    async def parse(self,config_file):
        output=[]
        with open(config_file, 'rt') as file:
            text = file.read()
        # Remove comments
        # Remove \r if found for some reason
        text = text.replace("\r", "")
        text_lines = text.split("\n")
        cleaned_text_lines = []
        for line in text_lines:
            if not line.startswith("//"):
                cleaned_text_lines.append(line)
        cleaned_text = "\n".join(cleaned_text_lines).strip()
        # Parse the text without lines beginning with //
        contents = json.loads(cleaned_text)["config"]
        for arr in contents:
            pair=[]
            for key, value in arr.items():
                pair.append(value)
            output.append(pair)
        return output
