import os
from .blitzbeaver import (
    DistanceMetricConfig,
    ElementType,
    MemoryConfig,
    RecordSchema,
    RecordScorerConfig,
    ResolverConfig,
    TrackerConfig,
    TrackingConfig,
)

DEFAULT_DISTANCE_METRIC_CONFIG = DistanceMetricConfig(
    metric="lv_opti",
    caching_threshold=4,
    use_sigmoid=False,
)

DEFAULT_RECORD_SCORER_CONFIG = RecordScorerConfig(
    record_scorer="average",
)

DEFAULT_RESOLVER_CONFIG = ResolverConfig(
    resolving_strategy="best-match",
)

DEFAULT_MEMORY_CONFIG = MemoryConfig(
    memory_strategy="median",
)

DEFAULT_MULTISTRING_MEMORY_CONFIG = MemoryConfig(
    memory_strategy="mw-median",
    multiword_threshold_match=0.6,
    multiword_distance_metric=DEFAULT_DISTANCE_METRIC_CONFIG,
)

DEFAULT_INTEREST_THRESHOLD = 0.6
DEFAULT_LIMIT_NO_MATCH_STREAK = 10


def _get_default_num_threads() -> int:
    """
    Returns the total number of available CPU cores.
    """
    return len(os.sched_getaffinity(0))


def _check_record_scorer_config(
    record_scorer_config: RecordScorerConfig, num_features: int
) -> None:
    if record_scorer_config.weights is not None:
        if len(record_scorer_config.weights) != num_features:
            raise ValueError(
                "The length of the RecordScorerConfig.weights must be equal to the number of features."
            )


def config(
    record_schema: RecordSchema,
    distance_metric_config: DistanceMetricConfig | None = None,
    record_scorer_config: RecordScorerConfig | None = None,
    resolver_config: ResolverConfig | None = None,
    memory_config: MemoryConfig | None = None,
    multistring_memory_config: MemoryConfig | None = None,
    interest_threshold: float | None = None,
    limit_no_match_streak: int | None = None,
    num_threads: int | None = None,
) -> TrackingConfig:
    """
    Builds a TrackingConfig object with the given configuration.

    This is a helper function, a TrackingConfig object can also be instantiated
    directly.

    Args:
        record_schema: The schema of the records to track.
        distance_metric_config: The configuration for the distance metric.
        record_scorer_config: The configuration for the record scorer.
        resolver_config: The configuration for the resolver.
        memory_config: The configuration for the "string" memory.
        multistring_memory_config: The configuration for the multi-string
            memory.
        interest_threshold: TrackerConfig.interest_threshold.
        limit_no_match_streak: interest_threshold.limit_no_match_streak
        num_threads: The number of threads to use.

    Returns:
        A TrackingConfig object with the given configuration.
    """

    if distance_metric_config is None:
        distance_metric_config = DEFAULT_DISTANCE_METRIC_CONFIG
    if record_scorer_config is None:
        record_scorer_config = DEFAULT_RECORD_SCORER_CONFIG
    if resolver_config is None:
        resolver_config = DEFAULT_RESOLVER_CONFIG
    if memory_config is None:
        memory_config = DEFAULT_MEMORY_CONFIG
    if multistring_memory_config is None:
        multistring_memory_config = DEFAULT_MULTISTRING_MEMORY_CONFIG
    if interest_threshold is None:
        interest_threshold = DEFAULT_INTEREST_THRESHOLD
    if limit_no_match_streak is None:
        limit_no_match_streak = DEFAULT_LIMIT_NO_MATCH_STREAK
    if num_threads is None:
        num_threads = _get_default_num_threads()

    memories = []
    for field_schema in record_schema.fields:
        if field_schema.dtype == ElementType.String:
            memories.append(memory_config)
        else:
            memories.append(multistring_memory_config)

    _check_record_scorer_config(record_scorer_config, len(record_schema.fields))

    return TrackingConfig(
        num_threads=num_threads,
        tracker=TrackerConfig(
            interest_threshold=interest_threshold,
            limit_no_match_streak=limit_no_match_streak,
            memories=memories,
            record_scorer=record_scorer_config,
        ),
        distance_metric=distance_metric_config,
        resolver=resolver_config,
    )
