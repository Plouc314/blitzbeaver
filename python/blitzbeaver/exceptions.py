class BlitzBeaverException(Exception):
    """
    Base exception for BlitzBeaver
    """


class InvalidConfigException(BlitzBeaverException):
    """
    Exception raised when the configuration is invalid
    """
