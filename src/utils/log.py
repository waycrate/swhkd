import sys
import time

class LOG_UTILS():
    def __init__(self):
        self.COLOR_RED="\033[1;31m"
        self.COLOR_GREEN="\033[1;32m"
        self.COLOR_YELLOW="\033[1;33m"
        self.COLOR_BLUE="\033[1;34m"
        self.COLOR_RESET="\033[0m"

    async def log_info(self, message:str):
        print(f"{self.COLOR_GREEN}[{time.ctime()}] INFO:{self.COLOR_RESET} {message}")

    async def log_error(self, message:str):
        print(f"{self.COLOR_RED}[{time.ctime()}] ERROR:{self.COLOR_RESET} {message}", file=sys.stderr)

    async def log_warn(self, message:str):
        print(f"{self.COLOR_YELLOW}[{time.ctime()}] WARN:{self.COLOR_RESET} {message}")
