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
        num_threads: int,
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
    use_sigmoid: bool
    lv_edit_weights: list[float] | None
    lv_substring_weight: float | None
    lv_multiword_separator: str | None

    def __init__(
        self,
        metric: DistanceMetric,
        caching_threshold: int,
        use_sigmoid: bool,
        lv_edit_weights: list[float] | None = None,
        lv_substring_weight: float | None = None,
        lv_multiword_separator: str | None = None,
    ) -> None: ...

class MemoryConfig:
    memory_strategy: MemoryStrategy
    multiword_threshold_match: float | None
    multiword_distance_metric: DistanceMetricConfig | None

    def __init__(
        self,
        memory_strategy: MemoryStrategy,
        multiword_threshold_match: float | None = None,
        multiword_distance_metric: DistanceMetricConfig | None = None,
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

class TrackerConfig:
    interest_threshold: float
    limit_no_match_streak: int
    memories: list[MemoryConfig]
    record_scorer: RecordScorerConfig

    def __init__(
        self,
        interest_threshold: float,
        limit_no_match_streak: int,
        memories: list[MemoryConfig],
        record_scorer: RecordScorerConfig,
    ) -> None: ...

class NormalizationConfig:
    threshold_cluster_match: float
    min_cluster_size: int
    distance_metric: DistanceMetricConfig

    def __init__(
        self,
        threshold_cluster_match: float,
        min_cluster_size: int,
        distance_metric: DistanceMetricConfig,
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

    def get_tracking_chain(self, id: ID) -> list[ChainNode]:
        """
        Internal method

        Builds the tracking chain with the given ID.
        """

# Diagnostics

class TrackerRecordDiagnostics:
    """
    Diagnostic information about a record,
    this contains internal information about the
    scoring process of the record by the tracker.
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

class TrackerFrameDiagnostics:
    """
    Diagnostic information about a frame,
    this contains internal information about the
    scoring process of the records in the frame
    by the tracker.
    """

    frame_idx: int
    """Index of the frame"""
    records: list[TrackerRecordDiagnostics]
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
    frames: list[TrackerFrameDiagnostics]
    """
    Diagnostic information for all the frames
    where the tracker is alive.
    """

class ResolvingDiagnostics:
    """
    Diagnostic information about the resolving process
    for a frame.

    Note: a record is considered matched by a tracker if
    its score is above the interest threshold.
    """

    histogram_record_matchs: list[int]
    """
    Histogram of the number of match for each record
    """
    histogram_tracker_matchs: list[int]
    """
    Histogram of the number of match for each tracker
    """

class Diagnostics:
    """
    Diagnostic information about the tracking process,
    this contains internal information about the
    scoring process of the trackers.
    """

    resolvings: list[ResolvingDiagnostics]
    """
    Diagnostic information about the resolving process
    for each frame.
    """

    def get_tracker(self, id: ID) -> TrackerDiagnostics | None:
        """
        Get the diagnostic information for the tracker with the given ID.

        Args:
            id: ID of the tracker

        Returns:
            Diagnostic information of the tracker or None if not found.
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

def execute_normalization_process(
    normalization_config: NormalizationConfig,
    record_schema: RecordSchema,
    tracking_graph: TrackingGraph,
    dataframes: list[pl.DataFrame],
) -> list[pl.DataFrame]:
    """
    Internal function

    Main entry point for the normalization process.
    """

def compute_median_word(words: list[str]) -> str | None:
    """
    Computes the median word from a list of words.

    Args:
        words: List of words

    Returns:
        The median word or None if the list is empty.
    """

def compute_words_clusters(
    words: list[str],
    distance_mertric_config: DistanceMetricConfig,
    threshold_match: float,
) -> list[list[str]]:
    """
    Computes the clusters of words from a list of words.

    Args:
        words: List of words
        distance_mertric_config: Distance metric configuration
        threshold_match: Threshold for a word to be considered a match
                        with a cluster.

    Returns:
        List of clusters of words
    """

def normalize_words(
    words: list[str | None],
    distance_mertric_config: DistanceMetricConfig,
    threshold_match: float,
    min_cluster_size: int,
) -> list[str | None]:
    """
    Normalizes a list of words using clustering.

    Args:
        words: List of words
        distance_mertric_config: Distance metric configuration
        threshold_match: Threshold for a word to be considered a match
                        with a cluster.
        min_cluster_size: Minimum size of a cluster to be considered

    Returns:
        List of normalized words
    """

# Genealogy

class GenealogyConfig:
    husband_name_idx: int
    wife_name_idx: int
    last_name_idx: int
    children_idx: int
    min_tracking_chain_length: int
    min_child_count: int
    search_last_frame_child: bool
    search_year_range: int
    matching_threshold: float

def execute_genealogy_process(
    genealogy_config: GenealogyConfig,
    distance_metric_config: DistanceMetricConfig,
    tracking_graph: TrackingGraph,
    dataframes: list[pl.DataFrame],
) -> list[str]: ...
