import polars as pl

from .blitzbeaver import (
    Diagnostics,
    FrameDiagnostics,
    RecordSchema,
    TrackingGraph as _TrackingGraph,
    ChainNode,
)
from .literals import ID, Element


class MaterializedTrackerFrame:
    """
    Materialized tracker frame

    Represents a frame in the lifespan of a tracker, it can be
    a frame where no record matched with this tracker.

    Attributes:
        frame_idx: Index of the frame in the tracking graph
        record_idx: Index of the record that matched with the tracker
            in the frame, if any.
        record: The record that matched with the tracker in the frame, if any.
        frame_diagnostic: Diagnostic information about the
            frame and the tracker.
    """

    def __init__(
        self,
        frame_idx: int,
        record_idx: int | None,
        record: list[Element] | None,
        frame_diagnostic: FrameDiagnostics,
    ) -> None:
        self.frame_idx = frame_idx
        self.record_idx = record_idx
        self.record = record
        self.frame_diagnostic = frame_diagnostic


class MaterializedTrackingChain:
    """
    Materialized tracking chain

    Represents a tracking chain, it is a list of materialized frames
    that represent the lifespan of the tracker.

    Attributes:
        frames: List of materialized frames
        matched_frames: Frames where a record matched with the tracker
        length: Length of the tracking chain
        lifespan: Lifespan of the tracker
    """

    def __init__(
        self,
        id: ID,
        frames: list[MaterializedTrackerFrame],
        record_schema: RecordSchema,
    ) -> None:
        self.id = id
        self.frames = frames
        self._schema = record_schema

    @property
    def matched_frames(self) -> list[MaterializedTrackerFrame]:
        """
        Frames where a record matched with the tracker
        """
        return [frame for frame in self.frames if frame.record_idx is not None]

    @property
    def length(self) -> int:
        """
        Length of the tracking chain, that is the number of frames
        for which a record matched with the tracker.
        """
        return len(self.matched_frames)

    @property
    def lifespan(self) -> int:
        """
        Lifespan of the tracker, that is the number of frames
        from the first matched frame to the last matched frame.
        """
        if len(self.matched_frames) == 0:
            return 0

        return self.matched_frames[-1].frame_idx - self.matched_frames[0].frame_idx + 1

    def as_dataframe(self) -> pl.DataFrame:
        """
        Returns the materialized tracking chain as a DataFrame

        Each row in the DataFrame represents a matching record in
        the tracking chain, the columns are the fields of the record
        schema with an additional column `"frame_idx"` that contains
        the index of the frame of the record.

        Returns:
            DataFrame containing the materialized tracking
        """
        records = []
        for frame in self.frames:
            if frame.record is not None:
                records.append([frame.frame_idx, *frame.record])

        return pl.DataFrame(
            records,
            schema=["frame_idx", *(field.name for field in self._schema.fields)],
            orient="row",
        )

    def __repr__(self) -> str:
        return f"MaterializedTrackingChain(id={self.id}, length={self.length}, lifespan={self.lifespan})"


class TrackingGraph:
    """
    Tracking graph

    Represents the tracking graph, comprising all tracking chains.
    It is the result of the tracking process, as such it can not be
    created directly, it is either returned by the tracking engine or
    loaded from a .beaver file.
    """

    def __init__(
        self,
        raw: _TrackingGraph,
        diagnostics: Diagnostics,
    ) -> None:
        self._raw = raw
        self.diagnostics = diagnostics

    @property
    def trackers_ids(self) -> list[ID]:
        """
        IDs of all trackers in the tracking graph.
        """
        return [id for id, _ in self._raw.root.outs]

    def _get_out_with_id(
        self, outs: list[tuple[ID, ChainNode]], id: ID
    ) -> tuple[ID, ChainNode] | None:
        for out in outs:
            if out[0] == id:
                return out
        return None

    def materialize_tracking_chain(
        self,
        id: ID,
        dataframes: list[pl.DataFrame],
        record_schema: RecordSchema,
    ) -> MaterializedTrackingChain:
        """
        Materializes a tracking chain

        Materializes a tracking chain given its ID, the dataframes
        containing the records and the record schema.

        This will generate a list of materialized frames for all frames
        in the tracker's lifespan.

        Args:
            id: ID of the tracker to materialize
            dataframes: List of DataFrames containing the records
            record_schema: Record schema

        Returns:
            Materialized tracking chain
        """

        # first get values of all node in the chain
        # this can be not continuous
        records = {}
        columns = [field.name for field in record_schema.fields]

        node = self._get_out_with_id(self._raw.root.outs, id)

        if node is None:
            raise ValueError(
                f"Tracking chain with ID {id} not found in the tracking graph"
            )

        while node is not None:
            id, ch = node
            records[ch.frame_idx] = (
                ch.record_idx,
                dataframes[ch.frame_idx].select(columns).row(ch.record_idx),
            )

            node = self._get_out_with_id(
                self._raw.matrix[ch.frame_idx][ch.record_idx].outs, node[0]
            )

        # then build the materialized frames
        # for each frame in the tracker's lifespan
        frames = []

        for frame in self.diagnostics.trackers[id].frames:
            matching_record_idx, record = records.get(frame.frame_idx, (None, None))
            frames.append(
                MaterializedTrackerFrame(
                    frame.frame_idx,
                    matching_record_idx,
                    record,
                    frame,
                )
            )

        return MaterializedTrackingChain(id, frames, record_schema)
