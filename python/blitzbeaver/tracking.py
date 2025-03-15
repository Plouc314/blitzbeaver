import polars as pl

from .blitzbeaver import (
    TrackingConfig,
    RecordSchema,
    execute_tracking_process,
)
from .logger import setup_logger, LogLevel
from .tracking_graph import TrackingGraph


def execute_tracking(
    tracking_config: TrackingConfig,
    record_schema: RecordSchema,
    dataframes: list[pl.DataFrame],
    log_level: LogLevel = "info",
) -> TrackingGraph:
    """
    Executes the tracking process, builds the tracking graph.

    This is the main entry point for the tracking process.

    Args:
        tracking_config: Tracking configuration
        record_schema: Record schema
        dataframes: List of DataFrames containing the records
        log_level: The log level to set the logger to, defaults to "info".
            This will not overwrite the logger if it has already been set up.

    Returns:
        The tracking graph built by the tracking process
    """
    setup_logger(log_level)

    raw_graph, diagnostics = execute_tracking_process(
        tracking_config,
        record_schema,
        dataframes,
    )

    return TrackingGraph(raw_graph, diagnostics)
