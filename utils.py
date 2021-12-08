class SWHKD_UTILS():
    """ 
    Helper Functions.
    """
    def __init__(self):
        self.COLOR_RED="\033[1;31m"
        self.COLOR_GREEN="\033[1;32m"
        self.COLOR_YELLOW="\033[1;33m"
        self.COLOR_BLUE="\033[1;34m"
        self.COLOR_RESET="\033[0m"

    def log_info(self, message:str):
        print(f"{self.COLOR_GREEN}INFO:{self.COLOR_RESET} {message}")

    def log_error(self, message:str):
        print(f"{self.COLOR_RED}ERROR:{self.COLOR_RESET} {message}")

    def log_warn(self, message:str):
        print(f"{self.COLOR_YELLOW}WARN:{self.COLOR_RESET} {message}")
