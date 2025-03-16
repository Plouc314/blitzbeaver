from typing import Literal

ID = tuple[int, int]

Element = str | list[str] | None

ResolvingStrategy = Literal["simple", "best-match"]
DistanceMetric = Literal[
    "lv",
    "lv_opti",
    "lv_edit",
    "lv_substring",
    "lv_multiword",
]
MemoryStrategy = Literal[
    "bruteforce",
    "mostfrequent",
    "median",
    "lsbruteforce",
    "lsmostfrequent",
    "lsmedian",
]
RecordScorer = Literal["average", "weighted-average", "weighted-quadratic"]
