# BlitzBeaver

BlitzBeaver is a Python package that allows for persons tracking accross historical records. It is desiged to work with noisy and incomplete data.

## Getting Started

### Prerequisites

- Python 3.10 or higher
- Rust edition 2021

### Development Setup

- Install the Python dependencies:

  ```bash
  pip3 install -r requirements.txt
  ```

> [!NOTE]
> It is recommended to use a virtual environment for Python dependencies ([venv](https://docs.python.org/3/library/venv.html)).

- To compile the Rust code, run:

  ```bash
  maturin develop
  ```

  Or in release mode:

  ```bash
  maturin develop -r
  ```

- To install the blitzbeaver package locally:
  ```bash
  pip3 install -e /path/to/blitzbeaver
  ```

## Records

The historical records given to BlitzBeaver are expected to be in a specific format.
The library used to work with the data is [polars](https://pola.rs/), it is similar to [pandas](https://pandas.pydata.org/) but has a better integration with Rust (as it is also written in Rust).

**Frame**  
Represents the records at a single point in time (ex: all records from year 1805 if the records are yearly).

**Record**  
Represents a single record (line) in the historical records.

**Element**  
Represents a single value in a record (ex: the name of a person, the birth date, etc.).

```python
import polars as pl

# example of an element
name = "Bob"

# example of a record
record = ["Bob", "Smith", "farmer"]

# example of a frame
frame = pl.DataFrame(
    {
        "name": ["Bob", "Alice"],
        "surname": ["Smith", "Johnson"],
        "occupation": ["farmer", "teacher"]
    }
)
```

### Record Schema

Represents the schema of a record. It is used to define the structure of the records in the historical records.
Each field in the schema corresponds to a column in the frame.

Elements can be of one of two types:

- `ElementType.String`: a single string value (ex: `"Bob"`)
- `ElementType.MultiStrings`: a list of strings (ex: `["Bob", "Alice"]`)

```python
import blitzbeaver as bb

record_schema = bb.RecordSchema(
    [
        bb.FieldSchema("address", bb.ElementType.String),
        bb.FieldSchema("firstname", bb.ElementType.String),
        bb.FieldSchema("lastname", bb.ElementType.String),
        bb.FieldSchema("origin", bb.ElementType.String),
        bb.FieldSchema("occupation", bb.ElementType.String),
        bb.FieldSchema("children", bb.ElementType.MultiStrings),
    ]
)
```

## Tracking

The tracking process attempts to match records across different frames, the results of this process are a list of tracking chains.
Each tracking chain represents a single entity (person), it is composed of a list of records that are believed to be the same entity at different points in time.

The results of the tracking process is actually not stored as a list of tracking chains, but rather as a graph (`TrackingGraph`) where each node is a record and each edge represents a link between two records.

### Tracker

The tracker is the component responsible for tracking a single entity across the frames, it gradually builds a tracking chain.

The tracker has a memory, it is responsible for producing the most representative values from the records it has seen so far.

```python
import blitzbeaver as bb

# reconstruct a tracking chain from the tracking graph
chain = graph.materialize_tracking_chain(tracker_id, dataframes, record_schema)

# display the tracking chain as a dataframe
chain.as_dataframe()

# outputs:
┌───────────┬─────────┬───────────┬──────────┬────────────┬────────────┬──────────────┐
│ frame_idx ┆ address ┆ firstname ┆ lastname ┆ origin     ┆ occupation ┆ children     │
╞═══════════╪═════════╪═══════════╪══════════╪════════════╪════════════╪══════════════╡
│ 0         ┆ bourg   ┆ clemont   ┆ rafford  ┆ anglais    ┆ lampiste   ┆ ["francois"] │
│ 1         ┆ bourg   ┆ lement    ┆ prafford ┆ null       ┆ null       ┆ null         │
│ 2         ┆ bourg   ┆ clement   ┆ trafford ┆ anglais    ┆ null       ┆ ["francois"] │
│ 3         ┆ bourg   ┆ clement   ┆ prafford ┆ anglais    ┆ rentier    ┆ ["francois"] │
│ 4         ┆ bourg   ┆ rement    ┆ grafford ┆ anglais    ┆ rentier    ┆ ["francois"] │
│ 5         ┆ boulg   ┆ clement   ┆ rafford  ┆ angleterre ┆ rentier    ┆ ["francois"] │
└───────────┴─────────┴───────────┴──────────┴────────────┴────────────┴──────────────┘
```

### Configuration

The tracking process takes a configuration that defines all the parameters of the tracking process.

Here is an example of a configuration:

```python
import blitzbeaver as bb

distance_metric_config = bb.DistanceMetricConfig(
    metric="lv_substring",
    caching_threshold=4,
    use_sigmoid=False,
    lv_substring_weight=0.5,
)
normal_memory_config = bb.MemoryConfig(
    memory_strategy="median",
)
multi_memory_config = bb.MemoryConfig(
    memory_strategy="mw-median",
    multiword_threshold_match=0.6,
    multiword_distance_metric=distance_metric_config,
)

config = bb.config(
    record_schema=record_schema,
    distance_metric_config=distance_metric_config,
    record_scorer_config=bb.RecordScorerConfig(
        record_scorer="average",
        weights=None,
        min_weight_ratio=None
    ),
    resolver_config=bb.ResolverConfig(
        resolving_strategy="best-match",
    ),
    memory_config=normal_memory_config,
    multistring_memory_config=multi_memory_config,
    interest_threshold=0.6,
    limit_no_match_streak=3,
    num_threads=10,
)
```

### Execution

The tracking process is executed as follows:

```python
import blitzbeaver as bb

tracking_graph = bb.execute_tracking(config, record_schema, dataframes)
```

### Diagnostics

The tracking process also returns some diagnostics information (`Diagnostics`).
These information provide insights on the tracking process, for example:

- The state of the memory of each tracker for each frame.
- The score of each record of interest for a tracker as well as the distances of each feature.

### Beaver file

The tracking graph and diagnostics information can be saved and loaded to/from a .beaver file.
The .beaver file is a binary file with a specific format.

```python
import blitzbeaver as bb

path_graph = "./graph.beaver"

# load the graph from a .beaver file
graph = bb.read_beaver(path_graph)

# save the graph to a .beaver file
bb.save_beaver(path_graph, graph)
```

## Normalization

Once computed, the tracking graph can be used to normalize values of the historical records. The idea being to use the link between multiple records of different frames of a tracking chain to correct errors and fill missing values.

```python
# the same tracker as above, with normalized values
chain = graph.materialize_tracking_chain(tracker_id, dataframes, record_schema, normalized_dataframes)

# display the tracking chain as a dataframe
chain.as_dataframe(normalized=True)

# outputs:
┌───────────┬─────────┬───────────┬──────────┬─────────┬────────────┬──────────────┐
│ frame_idx ┆ address ┆ firstname ┆ lastname ┆ origin  ┆ occupation ┆ children     │
╞═══════════╪═════════╪═══════════╪══════════╪═════════╪════════════╪══════════════╡
│ 0         ┆ bourg   ┆ clement   ┆ prafford ┆ anglais ┆ rentier    ┆ ["francois"] │
│ 1         ┆ bourg   ┆ clement   ┆ prafford ┆ anglais ┆ rentier    ┆ ["francois"] │
│ 2         ┆ bourg   ┆ clement   ┆ prafford ┆ anglais ┆ rentier    ┆ ["francois"] │
│ 3         ┆ bourg   ┆ clement   ┆ prafford ┆ anglais ┆ rentier    ┆ ["francois"] │
│ 4         ┆ bourg   ┆ clement   ┆ prafford ┆ anglais ┆ rentier    ┆ ["francois"] │
│ 5         ┆ bourg   ┆ clement   ┆ prafford ┆ anglais ┆ rentier    ┆ ["francois"] │
└───────────┴─────────┴───────────┴──────────┴─────────┴────────────┴──────────────┘
```

### Configuration

The normalization process takes a configuration:

```python
import blitzbeaver as bb

# the distance metric configuration to use to compute the distances
# between values during clustering
distance_metric_config: bb.DistanceMetricConfig = ...

normalization_config = bb.NormalizationConfig(
    threshold_cluster_match=0.5,
    min_cluster_size=2,
    distance_metric=distance_metric_config,
)
```

### Execution

The normalization process takes as argument the previously computed tracking graph, the historical records (`dataframes`), record schema and configuration.

It produces a list of dataframes: the normalized historical records.

```python
import blitzbeaver as bb

normalized_dataframes = bb.execute_normalization(
    normalization_config,
    record_schema,
    graph,
    dataframes,
)
```
