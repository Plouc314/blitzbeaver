from typing import Callable
import polars as pl

from .blitzbeaver import (
    Diagnostics,
    TrackerFrameDiagnostics,
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
            frame and the tracker. It is None for the first frame where the
            tracker is created.
    """

    def __init__(
        self,
        frame_idx: int,
        record_idx: int | None,
        record: list[Element] | None,
        normalized_record: list[Element] | None,
        frame_diagnostic: TrackerFrameDiagnostics | None,
    ) -> None:
        self.frame_idx = frame_idx
        self.record_idx = record_idx
        self.record = record
        self.normalized_record = normalized_record
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

    def as_dataframe(self, normalized: bool = False) -> pl.DataFrame:
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
                if normalized:
                    record = frame.normalized_record
                else:
                    record = frame.record
                records.append([frame.frame_idx, *record])

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
        self.trackers_ids = [id for id, _ in self._raw.root.outs]

    def materialize_tracking_chain(
        self,
        id: ID,
        dataframes: list[pl.DataFrame],
        record_schema: RecordSchema,
        normalized_dataframes: list[pl.DataFrame] | None = None,
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

        columns = [field.name for field in record_schema.fields]
        get_record: Callable[[ChainNode], list[Element]] = (
            lambda ch: dataframes[ch.frame_idx].select(columns).row(ch.record_idx)
        )
        get_normalized_record: Callable[[ChainNode], list[Element]] = lambda ch: (
            normalized_dataframes[ch.frame_idx].select(columns).row(ch.record_idx)
            if normalized_dataframes is not None
            else None
        )

        chain_nodes = self._raw.get_tracking_chain(id)

        if len(chain_nodes) == 0:
            raise ValueError(
                f"Tracking chain with ID {id} not found in the tracking graph"
            )

        map_frame_ch = {ch.frame_idx: ch for ch in chain_nodes}
        start_ch = chain_nodes[0]

        # build the materialized frames for each frame in the tracker's lifespan
        frames = [
            MaterializedTrackerFrame(
                frame_idx=start_ch.frame_idx,
                record_idx=start_ch.record_idx,
                record=get_record(start_ch),
                normalized_record=get_normalized_record(start_ch),
                frame_diagnostic=None,
            )
        ]

        tracker_diagnostics = self.diagnostics.get_tracker(id)
        for frame in tracker_diagnostics.frames:
            # there may or may not be a matching record for this frame
            ch = map_frame_ch.get(frame.frame_idx)
            frames.append(
                MaterializedTrackerFrame(
                    frame_idx=frame.frame_idx,
                    record_idx=ch.record_idx if ch is not None else None,
                    record=get_record(ch) if ch is not None else None,
                    normalized_record=(
                        get_normalized_record(ch) if ch is not None else None
                    ),
                    frame_diagnostic=frame,
                )
            )

        return MaterializedTrackingChain(id, frames, record_schema)
