## Configurations

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