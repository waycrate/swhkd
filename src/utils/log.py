import logging
import sys
from concurrent.futures import ThreadPoolExecutor, Executor

# https://stackoverflow.com/a/45843022/10291933
executor = ThreadPoolExecutor(max_workers=1)


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

    console_formatter = ColorfulFormatter()

    # https://stackoverflow.com/a/16066513/10291933
    stdout_handler = ExecutorStreamHandler(stream=sys.stdout,
                                           executor=executor)
    stdout_handler.setLevel(level=level)
    stdout_handler.addFilter(lambda record: record.levelno <= logging.INFO)
    stdout_handler.setFormatter(fmt=console_formatter)
    if stdout_handler not in logger.handlers:
        logger.addHandler(hdlr=stdout_handler)

    stderr_handler = ExecutorStreamHandler(stream=sys.stderr,
                                           executor=executor)
    stderr_handler.setLevel(level=logging.WARNING)
    stderr_handler.setFormatter(fmt=console_formatter)
    if stderr_handler not in logger.handlers:
        logger.addHandler(hdlr=stderr_handler)

    logger.debug(f"Created logger named {repr(name)} with level {repr(level)}")
    logger.debug(f"Handlers for {repr(name)}: {repr(logger.handlers)}")
    return logger


# https://stackoverflow.com/a/56944256/10291933
class ColorfulFormatter(logging.Formatter):
    red = "\033[1;31m"
    green = "\033[1;32m"
    yellow = "\033[1;33m"
    blue = "\033[1;34m"
    reset = "\033[0m"

    format = "%(asctime)s - %(name)s - %(levelname)s"

    formats = {
        logging.DEBUG: blue + format + reset + " - %(message)s",
        logging.INFO: green + format + reset + " - %(message)s",
        logging.WARNING: yellow + format + reset + " - %(message)s",
        logging.ERROR: red + format + reset + " - %(message)s",
        logging.CRITICAL: red + format + reset + " - %(message)s"
    }

    def format(self, record):
        log_fmt = self.formats.get(record.levelno)
        formatter = logging.Formatter(log_fmt)
        return formatter.format(record)


class ExecutorStreamHandler(logging.StreamHandler):
    def __init__(self, stream, executor: Executor):
        """
        A StreamHandler that sends the functions to call to a Executor.

        :param stream: The stream to write to.
        :param executor: The executor to submit to.
        """
        super().__init__(stream)
        self.executor = executor

    def emit(self, record: logging.LogRecord):
        """
        Emit a record to the executor.

        :param record: The record to emit eventually. (via the executor)
        """
        # super().emit(record)
        self.executor.submit(super().emit, record)

    def flush(self):
        """
        Have the executor flush the stream.
        """
        # super().flush()
        # return
        try:
            self.executor.submit(super().flush)
        except RuntimeError:
            # When the program stops we can't schedule the stream to be
            # flushed, so we do it synchronously
            super().flush()
