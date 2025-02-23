from enum import Enum, auto
import polars as pl

from .literals import ResolvingStrategy, DistanceMetric, TrackerType

class ElementType(Enum):
    String = auto()
    MultiStrings = auto()

class FieldSchema:
    name: str
    dtype: ElementType

    def __init__(self, name: str, dtype: ElementType) -> None: ...

class RecordSchema:
    fields: list[FieldSchema]

    def __init__(self, fields: list[FieldSchema]) -> None: ...

class TrackingConfig:
    tracker: "TrackerConfig"
    distance_metric: "DistanceMetricConfig"
    resolver: "ResolverConfig"

    def __init__(
        self,
        tracker: "TrackerConfig",
        distance_metric: "DistanceMetricConfig",
        resolver: "ResolverConfig",
    ) -> None: ...

class ResolverConfig:
    resolving_strategy: ResolvingStrategy

    def __init__(self, resolving_strategy: ResolvingStrategy) -> None: ...

class DistanceMetricConfig:
    metric: DistanceMetric
    caching_threshold: int

    def __init__(self, metric: DistanceMetric, caching_threshold: int) -> None: ...

class TrackerConfig:
    tracker_type: TrackerType
    simple_tracker: "SimpleTrackerConfig" | None

    def __init__(
        self,
        tracker_type: TrackerType,
        simple_tracker: "SimpleTrackerConfig" | None = None,
    ) -> None: ...

class SimpleTrackerConfig:
    interest_threshold: float

    def __init__(self, interest_threshold: float) -> None: ...

def test_tracking_engine(
    record_schema: RecordSchema,
    dataframes: list[pl.DataFrame],
    column_names: list[str],
) -> None: ...
def benchmark_distance_functions(
    values: pl.Series, value: str, num_runs: int, distance_function: str
) -> float: ...
def benchmark_feature_distance_calculator(
    values1: pl.Series,
    values2: pl.Series,
    num_runs: int,
    cache_dist_threshold: int,
    distance_function: str,
) -> tuple[int, int, int, int, float]: ...
def benchmark_feature_distance_calculator_second_pass(
    values1: pl.Series,
    values2: pl.Series,
    values3: pl.Series,
    num_runs: int,
    cache_dist_threshold: int,
    distance_function: str,
) -> tuple[int, int, int, int, float]: ...
def benchmark_feature_distance_calculator_multi_pass(
    values1: list[pl.Series],
    num_runs: int,
    cache_dist_threshold: int,
    distance_function: str,
) -> tuple[list[tuple[int, int, int, int]], float]: ...
