from enum import Enum, auto
import polars as pl

from .literals import (
    ResolvingStrategy,
    DistanceMetric,
    MemoryStrategy,
    RecordScorer,
    ID,
)

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
    num_threads: int
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
    interest_threshold: float
    memory_strategy: MemoryStrategy
    record_scorer: "RecordScorerConfig"

    def __init__(
        self,
        interest_threshold: float,
        memory_strategy: MemoryStrategy,
        record_scorer: "RecordScorerConfig",
    ) -> None: ...

class RecordScorerConfig:
    record_scorer: RecordScorer
    weights: list[float] | None

    def __init__(
        self, record_scorer: RecordScorer, weights: list[float] | None = None
    ) -> None: ...

class ChainNode:
    frame_idx: int
    record_idx: int

class GraphNode:
    outs: list[tuple[ID, ChainNode]]

class TrackingGraph:
    root: GraphNode
    matrix: list[list[GraphNode]]

    @staticmethod
    def from_bytes(bytes: bytes) -> "TrackingGraph": ...
    def to_bytes(self) -> bytes: ...

class EvalMetricChainLength:
    average: float
    median: float
    max: int
    min: int
    histogram: list[int]

class EvalMetricGraphProperties:
    records_match_ratios: list[float]
    trackers_match_ratios: list[float]
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
def compute_median_word(words: list[str]) -> str | None: ...
