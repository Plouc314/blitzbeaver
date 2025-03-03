from .blitzbeaver import (
    TrackingConfig,
    TrackerConfig,
    DistanceMetricConfig,
    ResolverConfig,
)
from .exceptions import InvalidConfigException

RESOLVING_STRATEGIES = ["simple"]
DISTANCE_METRICS = ["lv", "lvopti"]
TRACKER_TYPES = ["simple"]
MEMORY_STRATEGIES = [
    "bruteforce",
    "mostfrequent",
    "median",
    "lsbruteforce",
    "lsmostfrequent",
    "lsmedian",
]


def validate_tracker_config(tracker: TrackerConfig) -> None:
    if tracker.interest_threshold < 0 or tracker.interest_threshold > 1:
        raise InvalidConfigException(
            f"Invalid interest threshold: {tracker.interest_threshold}"
        )
    if tracker.memory_strategy not in MEMORY_STRATEGIES:
        raise InvalidConfigException(
            f"Invalid memory strategy: {tracker.memory_strategy}"
        )


def validate_distance_metric_config(
    distance_metric_config: DistanceMetricConfig,
) -> None:
    if distance_metric_config.metric not in DISTANCE_METRICS:
        raise InvalidConfigException(
            f"Invalid distance metric: {distance_metric_config.metric}"
        )


def validate_resolver_config(resolver_config: ResolverConfig) -> None:
    if resolver_config.resolving_strategy not in RESOLVING_STRATEGIES:
        raise InvalidConfigException(
            f"Invalid resolving strategy: {resolver_config.resolving_strategy}"
        )


def validate_tracking_config(config: TrackingConfig) -> None:
    """
    Validate the tracking configuration.

    Args:
        config: Tracking configuration to validate.

    Raises:
        InvalidConfigException: If the configuration is invalid
    """
    validate_tracker_config(config.tracker)
    validate_distance_metric_config(config.distance_metric)
    validate_resolver_config(config.resolver)
