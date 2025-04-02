from typing import Literal

ID = int

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
    "ls-bruteforce",
    "ls-mostfrequent",
    "ls-median",
    "mw-mostfrequent",
    "mw-median",
]
RecordScorer = Literal["average", "weighted-average", "weighted-quadratic"]
