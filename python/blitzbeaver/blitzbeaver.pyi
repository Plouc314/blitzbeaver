from enum import Enum, auto
import polars as pl

from .literals import (
    ResolvingStrategy,
    DistanceMetric,
    MemoryStrategy,
    RecordScorer,
    ID,
)

# Frame

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

# Config

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
    min_weight_ratio: float | None

    def __init__(
        self,
        record_scorer: RecordScorer,
        weights: list[float] | None = None,
        min_weight_ratio: float | None = None,
    ) -> None: ...

# Tracking graph

class ChainNode:
    """
    Internal class
    """

    frame_idx: int
    record_idx: int

class GraphNode:
    """
    Internal class
    """

    outs: list[tuple[ID, ChainNode]]

class TrackingGraph:
    """
    Internal class
    """

    root: GraphNode
    matrix: list[list[GraphNode]]

# Diagnostics

class RecordScoreDiagnostics:
    """
    Diagnostic information about a record,
    this contains internal information about the
    scoring process of the record.
    """

    record_idx: int
    """Index of the record in the frame"""
    record_score: float
    """Score of the record"""
    distances: list[float | None]
    """
    For each feature, distance retained between
    the record and the tracker memory.
    """

class FrameDiagnostics:
    """
    Diagnostic information about a frame,
    this contains internal information about the
    scoring process of the records in the frame.
    """

    frame_idx: int
    """Index of the frame"""
    records: list[RecordScoreDiagnostics]
    """
    Diagnostic information of the records considered of
    interest in the frame.
    """
    memory: list[list[str]]
    """
    For each feature, values of the memory of the tracker
    at this frame.
    """

class TrackerDiagnostics:
    """
    Diagnostic information about a tracker,
    this contains internal information about the
    scoring process of the tracker.
    """

    id: ID
    """ID of the tracker"""
    frames: list[FrameDiagnostics]
    """
    Diagnostic information for all the frames
    where the tracker is alive.
    """

class Diagnostics:
    """
    Diagnostic information about the tracking process,
    this contains internal information about the
    scoring process of the trackers.
    """

    trackers: dict[ID, TrackerDiagnostics]
    """
    Diagnostic information for all the trackers
    that were created during the tracking process.
    """

# Beaver

class BeaverFile:
    """
    Internal class

    Represents a .beaver file
    """

    @staticmethod
    def from_bytes(bytes: bytes) -> "BeaverFile": ...
    def to_bytes(self) -> bytes: ...
    def set_tracking_graph(self, graph: TrackingGraph) -> None: ...
    def take_tracking_graph(self) -> TrackingGraph: ...
    def set_diagnostics(self, diagnostics: Diagnostics) -> None: ...
    def take_diagnostics(self) -> Diagnostics: ...

# Evaluation

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

# API

def setup_logger(log_level: str) -> None:
    """
    Internal function

    Sets up the logger, this can only be called once.
    """

def execute_tracking_process(
    tracking_config: TrackingConfig,
    record_schema: RecordSchema,
    dataframes: list[pl.DataFrame],
) -> tuple[TrackingGraph, Diagnostics]:
    """
    Internal function

    Main entry point for the tracking process.
    """

def compute_median_word(words: list[str]) -> str | None:
    """
    Computes the median word from a list of words.

    Args:
        words: List of words

    Returns:
        The median word or None if the list is empty.
    """
