import polars as pl

class FieldSchema:
    name: str
    dtype: pl.DataType

    def __init__(self, name: str, dtype: pl.DataType) -> None: ...

class RecordSchema:
    fields: list[FieldSchema]

    def __init__(self, fields: list[FieldSchema]) -> None: ...

def benchmark_distance_functions(
    values: pl.Series, value: str, num_runs: int, distance_function: str
) -> float: ...
def benchmark_feature_distance_calculator(
    values1: pl.Series,
    values2: pl.Series,
    num_runs: int,
    cache_dist_threshold: int,
    distance_function: str,
) -> tuple[int, int, int, int, float]: ...
def benchmark_feature_distance_calculator_second_pass(
    values1: pl.Series,
    values2: pl.Series,
    values3: pl.Series,
    num_runs: int,
    cache_dist_threshold: int,
    distance_function: str,
) -> tuple[int, int, int, int, float]: ...
def benchmark_feature_distance_calculator_multi_pass(
    values1: list[pl.Series],
    num_runs: int,
    cache_dist_threshold: int,
    distance_function: str,
) -> tuple[list[tuple[int, int, int, int]], float]: ...
