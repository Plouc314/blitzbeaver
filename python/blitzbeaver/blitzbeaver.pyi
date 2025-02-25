from enum import Enum, auto
import polars as pl

from .literals import ResolvingStrategy, DistanceMetric, TrackerType, ID

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

class TrackingNode:
    outs: list[tuple[ID, int, int]]

class TrackingGraph:
    root: TrackingNode
    matrix: list[list[TrackingNode]]

    @staticmethod
    def from_bytes(bytes: bytes) -> "TrackingGraph": ...
    def to_bytes(self) -> bytes: ...

class EvalMetricChainLength:
    average: float
    median: float
    max: int
    min: int

class EvalMetricGraphProperties:
    match_ratios: list[float]
    conflict_ratios: list[float]

def evaluate_tracking_chain_length(graph: TrackingGraph) -> EvalMetricChainLength: ...
def evaluate_tracking_graph_properties(
    graph: TrackingGraph,
) -> EvalMetricGraphProperties: ...
def test_tracking_engine(
    tracking_config: TrackingConfig,
    record_schema: RecordSchema,
    dataframes: list[pl.DataFrame],
) -> TrackingGraph: ...
