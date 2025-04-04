import polars as pl

from .blitzbeaver import (
    NormalizationConfig,
    RecordSchema,
    execute_normalization_process,
)
from .tracking_graph import TrackingGraph


def execute_normalization(
    config: NormalizationConfig,
    record_schema: RecordSchema,
    tracking_graph: TrackingGraph,
    dataframes: list[pl.DataFrame],
) -> list[pl.DataFrame]:
    """
    Executes the normalization process.

    This is the main entry point for the normalization process.

    Args:
        config: Normalization configuration
        record_schema: Record schema
        tracking_graph: Tracking graph
        dataframes: List of DataFrames containing the records

    Returns:
        The normalized DataFrames
    """
    return execute_normalization_process(
        config,
        record_schema,
        tracking_graph._raw,
        dataframes,
    )
