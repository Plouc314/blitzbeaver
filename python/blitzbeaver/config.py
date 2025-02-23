from .blitzbeaver import (
    TrackingConfig,
    TrackerConfig,
    DistanceMetricConfig,
    ResolverConfig,
    SimpleTrackerConfig,
)
from .exceptions import InvalidConfigException

RESOLVING_STRATEGIES = ["simple"]
DISTANCE_METRICS = ["lv", "lvopti"]
TRACKER_TYPES = ["simple"]


def validate_simple_tracker_config(simple_tracker_config: SimpleTrackerConfig) -> None:
    if (
        simple_tracker_config.interest_threshold < 0
        or simple_tracker_config.interest_threshold > 1
    ):
        raise InvalidConfigException(
            f"Invalid interest threshold: {simple_tracker_config.interest_threshold}"
        )


def validate_tracker_config(tracker: TrackerConfig) -> None:
    if tracker.tracker_type not in TRACKER_TYPES:
        raise InvalidConfigException(f"Invalid tracker type: {tracker.tracker_type}")
    if tracker.tracker_type == "simple":
        if tracker.simple_tracker is None:
            raise InvalidConfigException(
                "Simple tracker config must be provided for simple tracker type"
            )
        validate_simple_tracker_config(tracker.simple_tracker)


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
