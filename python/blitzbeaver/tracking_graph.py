import polars as pl

from .blitzbeaver import TrackingGraph as _TrackingGraph, ChainNode
from .literals import ID


class TrackingGraph:
    """
    Tracking graph

    Represents the tracking graph, comprising all tracking chains.
    It is the result of the tracking process, as such it can not be
    created directly, it is either returned by the tracking engine or
    loaded from a .beaver file.
    """

    def __init__(self, raw: _TrackingGraph) -> None:
        self._raw = raw

    def save(self, filepath: str) -> None:
        """
        Save the tracking graph to a .beaver file.

        Args:
            filepath: Path to the file
        """
        with open(filepath, "wb") as file:
            file.write(self._raw.to_bytes())

    @staticmethod
    def load(filepath: str) -> "TrackingGraph":
        """
        Load a tracking graph from a .beaver file.

        Args:
            filepath: Path to the file
        """
        with open(filepath, "rb") as file:
            raw = _TrackingGraph.from_bytes(file.read())
        return TrackingGraph(raw)

    def _get_out_with_id(
        self, outs: list[tuple[ID, ChainNode]], id: ID
    ) -> tuple[ID, ChainNode] | None:
        for out in outs:
            if out[0] == id:
                return out
        return None

    def materialize_tracking_chain(
        self, id: ID, dataframes: list[pl.DataFrame]
    ) -> pl.DataFrame:
        """
        Builds a DataFrame containing all records in the tracking chain
        with the specified ID.

        Args:
            id: ID of the tracking chain
            dataframes: List of DataFrames containing the records

        Returns:
            DataFrame containing all records in the tracking chain
        """

        records = []

        node = self._get_out_with_id(self._raw.root.outs, id)

        if node is None:
            raise ValueError(
                f"Tracking chain with ID {id} not found in the tracking graph"
            )

        while node is not None:
            id, ch = node
            records.append(dataframes[ch.frame_idx][ch.record_idx])
            node = self._get_out_with_id(
                self._raw.matrix[ch.frame_idx][ch.record_idx].outs, node[0]
            )

        return pl.concat(records)
