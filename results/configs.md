# Configurations

## Brute-force pass

### Record Schema

```python
record_schema = bb.RecordSchema(
    [
        bb.FieldSchema("nom_rue", bb.ElementType.String),
        bb.FieldSchema("chef_prenom", bb.ElementType.String),
        bb.FieldSchema("chef_nom", bb.ElementType.String),
        bb.FieldSchema("chef_origine", bb.ElementType.String),
        bb.FieldSchema("epouse_nom", bb.ElementType.String),
        bb.FieldSchema("chef_vocation", bb.ElementType.String),
        bb.FieldSchema("enfants_chez_parents_prenom", bb.ElementType.MultiStrings),
    ]
)
```

### Distance metrics

Total number of distance metrics: 6

```python
caching_threshold = 4

lv_substring_weights = [0.2, 0.4, 0.6, 0.8, 1.0]

distance_metric_lv_opti = bb.DistanceMetricConfig(
    metric="lv_opti",
    caching_threshold=caching_threshold,
    use_sigmoid=False,
)

distance_metrics = [distance_metric_lv_opti] + [
    bb.DistanceMetricConfig(
        metric="lv_substring",
        caching_threshold=caching_threshold,
        use_sigmoid=False,
        lv_substring_weight=w,
    )
    for w in lv_substring_weights
]
```

### Record scorers

Total number of record scorers: 10

```python
record_scorer_average = bb.RecordScorerConfig(
    record_scorer="average",
    weights=None,
    min_weight_ratio=None,
)

min_weight_ratios = [0.6, 0.8, 1.0]
weights = [
    [
        0.2,
        0.25,
        0.25,
        0.25,
        0.15,
        0.2,
        0.1,
    ],
    [
        0.1,
        0.3,
        0.3,
        0.3,
        0.1,
        0.1,
        0.1,
    ],
    [
        0.1,
        0.5,
        0.5,
        0.5,
        0.1,
        0.1,
        0.1,
    ],
]


record_scorer_weight = [
    bb.RecordScorerConfig(
        record_scorer="weighted-average",
        weights=w,
        min_weight_ratio=ratio,
    )
    for w in weights
    for ratio in min_weight_ratios
]

record_scorers = [record_scorer_average] + record_scorer_weight
```

### Resolvers

Total number of resolvers: 1

```python
resolver_config = bb.ResolverConfig(
    resolving_strategy="best-match",
)
```

### Memory strategies

Total number of memory strategies: 3
Total number of multiword memory strategies: 4

```python
memory_strategies = [
    "mostfrequent",
    "median",
    "ls-median",
]
memory_configs = [bb.MemoryConfig(memory_strategy=m) for m in memory_strategies]

multi_word_thresholds = [0.2, 0.4, 0.6, 0.8]

multistring_memory_config = [
    bb.MemoryConfig(
        memory_strategy="mw-median",
        multiword_threshold_match=t,
        multiword_distance_metric=distance_metric_lv_opti,
    )
    for t in multi_word_thresholds
]
```

### Interest thresholds

Total number of interest thresholds: 4

```python
interest_thresholds = [0.2, 0.4, 0.6, 0.8]
```

### Combinations

Total number of configurations: 2880

```python
configs = [
    bb.config(
        record_schema=record_schema,
        distance_metric_config=d,
        record_scorer_config=r,
        resolver_config=resolver_config,
        memory_config=m,
        multistring_memory_config=mm,
        interest_threshold=t,
        limit_no_match_streak=4,
        num_threads=17,
    )
    for d in distance_metrics
    for r in record_scorers
    for m in memory_configs
    for mm in multistring_memory_config
    for t in interest_thresholds
]
```

## Second pass

Test long-short term most-frequent vs long-short term median memory: median performs better
Test min weight ratio: 0.7 conclusive
Test interest threshold below 0.8 (0.7, 0.75): not conclusive

```python
caching_threshold = 4

lv_substring_weights = [0.4, 0.6, 0.8]

distance_metric_lv_opti = bb.DistanceMetricConfig(
    metric="lv_opti",
    caching_threshold=caching_threshold,
    use_sigmoid=False,
)

distance_metrics = [
    bb.DistanceMetricConfig(
        metric="lv_substring",
        caching_threshold=caching_threshold,
        use_sigmoid=False,
        lv_substring_weight=w,
    )
    for w in lv_substring_weights
]

min_weight_ratios = [0.7]
weights = [
    [
        0.2,
        0.25,
        0.25,
        0.25,
        0.15,
        0.2,
        0.1,
    ],
    [
        0.15,
        0.20,
        0.25,
        0.15,
        0.05,
        0.15,
        0.05,
    ],
    [
        0.3,
        0.4,
        0.5,
        0.3,
        0.1,
        0.2,
        0.1,
    ],
]


record_scorers = [
    bb.RecordScorerConfig(
        record_scorer="weighted-average",
        weights=w,
        min_weight_ratio=ratio,
    )
    for w in weights
    for ratio in min_weight_ratios
]

resolver_config = bb.ResolverConfig(
    resolving_strategy="best-match",
)

memory_strategies = [
    "ls-mostfrequent",
    "ls-median",
]
memory_configs = [bb.MemoryConfig(memory_strategy=m) for m in memory_strategies]

multistring_memory_config = [
    bb.MemoryConfig(
        memory_strategy="mw-median",
        multiword_threshold_match=0.8,
        multiword_distance_metric=distance_metric_lv_opti,
    )
]

thresholds = [0.7, 0.75, 0.8]
configs = [
    bb.config(
        record_schema=record_schema_base,
        distance_metric_config=d,
        record_scorer_config=r,
        resolver_config=resolver_config,
        memory_config=m,
        multistring_memory_config=mm,
        interest_threshold=t,
        limit_no_match_streak=4,
        num_threads=17,
    )
    for d in distance_metrics
    for r in record_scorers
    for m in memory_configs
    for mm in multistring_memory_config
    for t in thresholds
]
```

## Third pass

Test interest threshold above 0.8 (0.83, 0.85): not conclusive
Test more lv substring weight: 0.7 (& 0.5) conclusive

```python
caching_threshold = 4

lv_substring_weights = [0.4, 0.5, 0.6, 0.7, 0.8]

distance_metric_lv_opti = bb.DistanceMetricConfig(
    metric="lv_opti",
    caching_threshold=caching_threshold,
    use_sigmoid=False,
)

distance_metrics = [
    bb.DistanceMetricConfig(
        metric="lv_substring",
        caching_threshold=caching_threshold,
        use_sigmoid=False,
        lv_substring_weight=w,
    )
    for w in lv_substring_weights
]

min_weight_ratios = [0.7, 0.75]
weights = [
    [
        0.2,
        0.25,
        0.25,
        0.25,
        0.15,
        0.2,
        0.1,
    ],
    [
        0.20,
        0.25,
        0.30,
        0.20,
        0.05,
        0.20,
        0.05,
    ],
    [
        0.2,
        0.3,
        0.3,
        0.2,
        0.1,
        0.2,
        0.2,
    ],
]

record_scorers = [
    bb.RecordScorerConfig(
        record_scorer="weighted-average",
        weights=w,
        min_weight_ratio=ratio,
    )
    for w in weights
    for ratio in min_weight_ratios
]

resolver_config = bb.ResolverConfig(
    resolving_strategy="best-match",
)

memory_strategies = [
    "ls-median",
]
memory_configs = [bb.MemoryConfig(memory_strategy=m) for m in memory_strategies]

multistring_memory_config = [
    bb.MemoryConfig(
        memory_strategy="mw-median",
        multiword_threshold_match=0.8,
        multiword_distance_metric=distance_metric_lv_opti,
    )
]

thresholds = [0.8, 0.83, 0.85]
configs = [
    bb.config(
        record_schema=record_schema_base,
        distance_metric_config=d,
        record_scorer_config=r,
        resolver_config=resolver_config,
        memory_config=m,
        multistring_memory_config=mm,
        interest_threshold=t,
        limit_no_match_streak=4,
        num_threads=17,
    )
    for d in distance_metrics
    for r in record_scorers
    for m in memory_configs
    for mm in multistring_memory_config
    for t in thresholds
]
```

## Fourth pass

Interest threshold of 0.78-0.8 seems to be the best
Lv substring weight of 0.6, 0.7, 0.8 seems equivalent
Min weight ratio of 0.7-0.73 seems to be the best
Best weights: `[0.2, 0.25, 0.25, 0.25, 0.15, 0.15, 0.15]`


```python
caching_threshold = 4

lv_substring_weights = [0.6, 0.7, 0.8]

distance_metric_lv_opti = bb.DistanceMetricConfig(
    metric="lv_opti",
    caching_threshold=caching_threshold,
    use_sigmoid=False,
)

distance_metrics = [
    bb.DistanceMetricConfig(
        metric="lv_substring",
        caching_threshold=caching_threshold,
        use_sigmoid=False,
        lv_substring_weight=w,
    )
    for w in lv_substring_weights
]

min_weight_ratios = [0.7, 0.73]
weights = [
    [
        0.2,
        0.25,
        0.25,
        0.25,
        0.15,
        0.2,
        0.1,
    ],
    [
        0.2,
        0.25,
        0.25,
        0.25,
        0.15,
        0.15,
        0.15,
    ],
    [
        0.15,
        0.25,
        0.25,
        0.25,
        0.15,
        0.2,
        0.2,
    ],
]

record_scorers = [
    bb.RecordScorerConfig(
        record_scorer="weighted-average",
        weights=w,
        min_weight_ratio=ratio,
    )
    for w in weights
    for ratio in min_weight_ratios
]

resolver_config = bb.ResolverConfig(
    resolving_strategy="best-match",
)

memory_strategies = [
    "ls-median",
]
memory_configs = [bb.MemoryConfig(memory_strategy=m) for m in memory_strategies]

multistring_memory_config = [
    bb.MemoryConfig(
        memory_strategy="mw-median",
        multiword_threshold_match=0.8,
        multiword_distance_metric=distance_metrics[1],
    )
]

thresholds = [0.8, 0.82, 0.78]
configs = [
    bb.config(
        record_schema=record_schema_base,
        distance_metric_config=d,
        record_scorer_config=r,
        resolver_config=resolver_config,
        memory_config=m,
        multistring_memory_config=mm,
        interest_threshold=t,
        limit_no_match_streak=4,
        num_threads=17,
    )
    for d in distance_metrics
    for r in record_scorers
    for m in memory_configs
    for mm in multistring_memory_config
    for t in thresholds
]
```

## Fifth pass

Best weights by far: `[0.15, 0.25, 0.25, 0.20, 0.15, 0.15, 0.15]`

```python
caching_threshold = 4

lv_substring_weights = [0.6, 0.7, 0.8]

distance_metric_lv_opti = bb.DistanceMetricConfig(
    metric="lv_opti",
    caching_threshold=caching_threshold,
    use_sigmoid=False,
)

distance_metrics = [
    bb.DistanceMetricConfig(
        metric="lv_substring",
        caching_threshold=caching_threshold,
        use_sigmoid=False,
        lv_substring_weight=w,
    )
    for w in lv_substring_weights
]

min_weight_ratios = [0.7, 0.71, 0.72]
weights = [
    [
        0.20,
        0.25,
        0.25,
        0.25,
        0.15,
        0.15,
        0.15,
    ],
    [
        0.20,
        0.25,
        0.30,
        0.25,
        0.15,
        0.15,
        0.15,
    ],
    [
        0.15,
        0.25,
        0.25,
        0.20,
        0.15,
        0.15,
        0.15,
    ],
]

record_scorers = [
    bb.RecordScorerConfig(
        record_scorer="weighted-average",
        weights=w,
        min_weight_ratio=ratio,
    )
    for w in weights
    for ratio in min_weight_ratios
]

resolver_config = bb.ResolverConfig(
    resolving_strategy="best-match",
)

memory_strategies = [
    "ls-median",
]
memory_configs = [bb.MemoryConfig(memory_strategy=m) for m in memory_strategies]

multistring_memory_config = [
    bb.MemoryConfig(
        memory_strategy="mw-median",
        multiword_threshold_match=0.8,
        multiword_distance_metric=distance_metrics[1],
    )
]

thresholds = [0.79]
configs = [
    bb.config(
        record_schema=record_schema_base,
        distance_metric_config=d,
        record_scorer_config=r,
        resolver_config=resolver_config,
        memory_config=m,
        multistring_memory_config=mm,
        interest_threshold=t,
        limit_no_match_streak=4,
        num_threads=17,
    )
    for d in distance_metrics
    for r in record_scorers
    for m in memory_configs
    for mm in multistring_memory_config
    for t in thresholds
]
```

## Sixth pass

Weights: test variations of `[0.15, 0.25, 0.25, 0.20, 0.15, 0.15, 0.15]`
Interest threshold: 0.79 seems to be the best

```python
caching_threshold = 4

lv_substring_weights = [0.6, 0.7, 0.8]

distance_metric_lv_opti = bb.DistanceMetricConfig(
    metric="lv_opti",
    caching_threshold=caching_threshold,
    use_sigmoid=False,
)

distance_metrics = [
    bb.DistanceMetricConfig(
        metric="lv_substring",
        caching_threshold=caching_threshold,
        use_sigmoid=False,
        lv_substring_weight=w,
    )
    for w in lv_substring_weights
]

min_weight_ratios = [0.7]
weights = [
    [
        0.15,
        0.25,
        0.25,
        0.20,
        0.15,
        0.15,
        0.15,
    ],
    [
        0.15,
        0.25,
        0.30,
        0.20,
        0.15,
        0.15,
        0.15,
    ],
    [
        0.10,
        0.25,
        0.25,
        0.20,
        0.15,
        0.15,
        0.15,
    ],
    [
        0.15,
        0.25,
        0.25,
        0.15,
        0.15,
        0.15,
        0.15,
    ],
    [
        0.15,
        0.25,
        0.30,
        0.15,
        0.15,
        0.15,
        0.15,
    ],
]

record_scorers = [
    bb.RecordScorerConfig(
        record_scorer="weighted-average",
        weights=w,
        min_weight_ratio=ratio,
    )
    for w in weights
    for ratio in min_weight_ratios
]

resolver_config = bb.ResolverConfig(
    resolving_strategy="best-match",
)

memory_strategies = [
    "ls-median",
]
memory_configs = [bb.MemoryConfig(memory_strategy=m) for m in memory_strategies]

multistring_memory_config = [
    bb.MemoryConfig(
        memory_strategy="mw-median",
        multiword_threshold_match=0.8,
        multiword_distance_metric=distance_metrics[1],
    )
]

thresholds = [0.79, 0.8]
configs = [
    bb.config(
        record_schema=record_schema_base,
        distance_metric_config=d,
        record_scorer_config=r,
        resolver_config=resolver_config,
        memory_config=m,
        multistring_memory_config=mm,
        interest_threshold=t,
        limit_no_match_streak=4,
        num_threads=17,
    )
    for d in distance_metrics
    for r in record_scorers
    for m in memory_configs
    for mm in multistring_memory_config
    for t in thresholds
]
```
