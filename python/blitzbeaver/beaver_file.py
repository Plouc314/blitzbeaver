from .blitzbeaver import BeaverFile
from .tracking_graph import TrackingGraph
from .exceptions import InvalidBeaverFileException


def read_beaver(filepath: str) -> TrackingGraph:
    """
    Reads a .beaver file

    Args:
        filepath: Path to the .beaver file

    Returns:
        The tracking graph contained in the .beaver file.

    Raises:
        InvalidBeaverFileException: If the file is not a valid .beaver file.
    """
    try:
        with open(filepath, "rb") as file:
            beaver_file = BeaverFile.from_bytes(file.read())

        tracking_graph = TrackingGraph(
            beaver_file.take_tracking_graph(),
            beaver_file.take_diagnostics(),
        )
    except ValueError as e:
        raise InvalidBeaverFileException(str(e))

    return tracking_graph


def save_beaver(
    filepath: str,
    tracking_graph: TrackingGraph,
) -> None:
    """
    Saves a tracking graph to a .beaver file

    Args:
        filepath: Path to the .beaver file
        tracking_graph: Tracking graph to save
    """
    beaver_file = BeaverFile()
    beaver_file.set_tracking_graph(tracking_graph._raw)
    beaver_file.set_diagnostics(tracking_graph.diagnostics)

    with open(filepath, "wb") as file:
        file.write(beaver_file.to_bytes())
