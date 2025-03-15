class BlitzBeaverException(Exception):
    """
    Base exception for BlitzBeaver
    """


class InvalidConfigException(BlitzBeaverException):
    """
    Exception raised when the configuration is invalid
    """


class InvalidBeaverFileException(BlitzBeaverException):
    """
    Exception raised when the beaver file is invalid
    """
