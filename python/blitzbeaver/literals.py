from typing import Literal

ID = tuple[int, int]

ResolvingStrategy = Literal["simple"]
DistanceMetric = Literal["lv", "lvopti"]
TrackerType = Literal["simple"]
MemoryStrategy = Literal[
    "bruteforce",
    "mostfrequent",
    "median",
    "lsbruteforce",
    "lsmostfrequent",
    "lsmedian",
]
