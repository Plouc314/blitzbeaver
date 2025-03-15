from typing import Literal

ID = tuple[int, int]

Element = str | list[str] | None

ResolvingStrategy = Literal["simple", "best-match"]
DistanceMetric = Literal["lv", "lvopti"]
MemoryStrategy = Literal[
    "bruteforce",
    "mostfrequent",
    "median",
    "lsbruteforce",
    "lsmostfrequent",
    "lsmedian",
]
RecordScorer = Literal["average", "weighted-average", "weighted-quadratic"]
