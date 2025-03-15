from typing import Literal
from .blitzbeaver import setup_logger as _setup_logger

LogLevel = Literal[
    "trace",
    "debug",
    "info",
    "warn",
    "error",
]

logger_setup = False


def setup_logger(log_level: LogLevel = "info") -> None:
    """
    Sets up the logger for the blitzbeaver package.

    This is automatically before use of the package if not called manually.

    Args:
        log_level: The log level to set the logger to, defaults to "info".
    """
    global logger_setup

    if not logger_setup:
        logger_setup = True
        _setup_logger(log_level)
