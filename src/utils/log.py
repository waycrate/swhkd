import logging


def create_logger(name: str, level: int = logging.DEBUG) -> logging.Logger:
    """
    A simple function to create a logger. You would typically put this right
    under all the other modules you imported.

    And then call `logger.debug()`, `logger.info()`, `logger.warning()`,
    `logger.error()`, `logger.critical()`, and
    `logger.exception` everywhere in that module.

    :param name: A string with the logger name.
    :param level: A integer with the logger level. Defaults to logging.DEBUG.
    :return: A logging.Logger which you can use as a regular logger.
    """
    logger = logging.getLogger(name=name)
    logger.setLevel(level=level)
    logger.propagate = False
    console_handler = AsyncStreamHandler()
    console_handler.setLevel(level=level)
    console_formatter = logging.Formatter("%(asctime)s - %(name)s - "
                                          "%(levelname)s - %(message)s")
    console_handler.setFormatter(fmt=console_formatter)
    if console_handler not in logger.handlers:
        logger.addHandler(hdlr=console_handler)
    logger.debug(f"Created logger named {repr(name)} with level {repr(level)}")
    logger.debug(f"Handlers for {repr(name)}: {repr(logger.handlers)}")
    return logger


class AsyncStreamHandler(logging.StreamHandler):
    """
    The same exact thing as a StreamHandler in the logging module, but the emit
    function is defined asynchronously.
    """

    def __init__(self):
        super().__init__()

    async def emit(self, record: logging.LogRecord):
        """
        Does the exact same thing as a regular StreamHandler, but wrapped in an
        asynchronous function.

        :param record: The record to emit.
        """
        super().emit(record)


# class LOG_UTILS():
#     def __init__(self):
#         self.COLOR_RED="\033[1;31m"
#         self.COLOR_GREEN="\033[1;32m"
#         self.COLOR_YELLOW="\033[1;33m"
#         self.COLOR_BLUE="\033[1;34m"
#         self.COLOR_RESET="\033[0m"
#
#     async def log_info(self, message:str):
#         print(f"{self.COLOR_GREEN}[{time.ctime()}] INFO:{self.COLOR_RESET} {message}")
#
#     async def log_error(self, message:str):
#         print(f"{self.COLOR_RED}[{time.ctime()}] ERROR:{self.COLOR_RESET} {message}", file=sys.stderr)
#
#     async def log_warn(self, message:str):
#         print(f"{self.COLOR_YELLOW}[{time.ctime()}] WARN:{self.COLOR_RESET} {message}")
