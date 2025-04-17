from .blitzbeaver import (
    TrackingConfig,
    TrackerConfig,
    DistanceMetricConfig,
    ResolverConfig,
    RecordScorerConfig,
    MemoryConfig,
)
from .exceptions import InvalidConfigException


def serialize_distance_metric_config(c: DistanceMetricConfig) -> dict:
    """
    Serializes the DistanceMetricConfig object to a dictionary.

    Args:
        c: The DistanceMetricConfig object to serialize.

    Returns:
        A dictionary representation of the DistanceMetricConfig object.
    """
    return {
        "metric": c.metric,
        "caching_threshold": c.caching_threshold,
        "use_sigmoid": c.use_sigmoid,
        "lv_edit_weights": c.lv_edit_weights,
        "lv_substring_weight": c.lv_substring_weight,
        "lv_multiword_separator": c.lv_multiword_separator,
    }


def serialize_record_scorer_config(c: RecordScorerConfig) -> dict:
    """
    Serializes the RecordScorerConfig object to a dictionary.

    Args:
        c: The RecordScorerConfig object to serialize.

    Returns:
        A dictionary representation of the RecordScorerConfig object.
    """
    return {
        "record_scorer": c.record_scorer,
        "weights": c.weights,
        "min_weight_ratio": c.min_weight_ratio,
    }


def serialize_memory_config(c: MemoryConfig) -> dict:
    """
    Serializes the MemoryConfig object to a dictionary.

    Args:
        c: The MemoryConfig object to serialize.

    Returns:
        A dictionary representation of the MemoryConfig object.
    """
    return {
        "memory_strategy": c.memory_strategy,
        "multiword_threshold_match": c.multiword_threshold_match,
        "multiword_distance_metric": (
            None
            if c.multiword_distance_metric is None
            else serialize_distance_metric_config(c.multiword_distance_metric)
        ),
    }


def serialize_resolver_config(c: ResolverConfig) -> dict:
    """
    Serializes the ResolverConfig object to a dictionary.

    Args:
        c: The ResolverConfig object to serialize.

    Returns:
        A dictionary representation of the ResolverConfig object.
    """
    return {"resolving_strategy": c.resolving_strategy}


def serialize_tracker_config(c: TrackerConfig) -> dict:
    """
    Serializes the TrackerConfig object to a dictionary.

    Args:
        c: The TrackerConfig object to serialize.

    Returns:
        A dictionary representation of the TrackerConfig object.
    """
    return {
        "interest_threshold": c.interest_threshold,
        "limit_no_match_streak": c.limit_no_match_streak,
        "record_scorer": serialize_record_scorer_config(c.record_scorer),
        "memories": [serialize_memory_config(a) for a in c.memories],
    }


def serialize_tracking_config(c: TrackingConfig) -> dict:
    """
    Serializes the TrackingConfig object to a dictionary.

    Args:
        c: The TrackingConfig object to serialize.

    Returns:
        A dictionary representation of the TrackingConfig object.
    """
    return {
        "num_threads": c.num_threads,
        "tracker": serialize_tracker_config(c.tracker),
        "distance_metric": serialize_distance_metric_config(c.distance_metric),
        "resolver": serialize_resolver_config(c.resolver),
    }


def deserialize_distance_metric_config(d: dict) -> DistanceMetricConfig:
    """
    Deserializes a dictionary to a DistanceMetricConfig object.

    Args:
        d: The dictionary to deserialize.

    Returns:
        A DistanceMetricConfig object.
    """
    try:
        return DistanceMetricConfig(
            metric=d["metric"],
            caching_threshold=d["caching_threshold"],
            use_sigmoid=d["use_sigmoid"],
            lv_edit_weights=d.get("lv_edit_weights"),
            lv_substring_weight=d.get("lv_substring_weight"),
            lv_multiword_separator=d.get("lv_multiword_separator"),
        )
    except KeyError as e:
        raise InvalidConfigException(f"Missing key in DistanceMetricConfig: {e}")


def deserialize_record_scorer_config(d: dict) -> RecordScorerConfig:
    """
    Deserializes a dictionary to a RecordScorerConfig object.

    Args:
        d: The dictionary to deserialize.

    Returns:
        A RecordScorerConfig object.
    """
    try:
        return RecordScorerConfig(
            record_scorer=d["record_scorer"],
            weights=d.get("weights"),
            min_weight_ratio=d.get("min_weight_ratio"),
        )
    except KeyError as e:
        raise InvalidConfigException(f"Missing key in RecordScorerConfig: {e}")


def deserialize_memory_config(d: dict) -> MemoryConfig:
    """
    Deserializes a dictionary to a MemoryConfig object.

    Args:
        d: The dictionary to deserialize.

    Returns:
        A MemoryConfig object.
    """
    try:
        return MemoryConfig(
            memory_strategy=d["memory_strategy"],
            multiword_threshold_match=d.get("multiword_threshold_match"),
            multiword_distance_metric=(
                None
                if d.get("multiword_distance_metric") is None
                else deserialize_distance_metric_config(d["multiword_distance_metric"])
            ),
        )
    except KeyError as e:
        raise InvalidConfigException(f"Missing key in MemoryConfig: {e}")


def deserialize_resolver_config(d: dict) -> ResolverConfig:
    """
    Deserializes a dictionary to a ResolverConfig object.

    Args:
        d: The dictionary to deserialize.

    Returns:
        A ResolverConfig object.
    """
    try:
        return ResolverConfig(resolving_strategy=d["resolving_strategy"])
    except KeyError as e:
        raise InvalidConfigException(f"Missing key in ResolverConfig: {e}")


def deserialize_tracker_config(d: dict) -> TrackerConfig:
    """
    Deserializes a dictionary to a TrackerConfig object.

    Args:
        d: The dictionary to deserialize.

    Returns:
        A TrackerConfig object.
    """
    try:
        return TrackerConfig(
            interest_threshold=d["interest_threshold"],
            limit_no_match_streak=d["limit_no_match_streak"],
            record_scorer=deserialize_record_scorer_config(d["record_scorer"]),
            memories=[deserialize_memory_config(a) for a in d["memories"]],
        )
    except KeyError as e:
        raise InvalidConfigException(f"Missing key in TrackerConfig: {e}")


def deserialize_tracking_config(d: dict) -> TrackingConfig:
    """
    Deserializes a dictionary to a TrackingConfig object.

    Args:
        d: The dictionary to deserialize.

    Returns:
        A TrackingConfig object.
    """
    try:
        return TrackingConfig(
            num_threads=d["num_threads"],
            tracker=deserialize_tracker_config(d["tracker"]),
            distance_metric=deserialize_distance_metric_config(d["distance_metric"]),
            resolver=deserialize_resolver_config(d["resolver"]),
        )
    except KeyError as e:
        raise InvalidConfigException(f"Missing key in TrackingConfig: {e}")
