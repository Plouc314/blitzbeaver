use crate::{api, id::ID};

/// Follows a chain of tracking nodes and calls the given on_tracking_node function for each node.
fn follow_chain(
    graph: &api::TrackingGraph,
    id: ID,
    on_tracking_node: &mut dyn FnMut(&api::TrackingNode),
) {
    let mut node = graph.root.outs.iter().find(|o| o.0 == id);

    loop {
        match node {
            Some((id, frame_idx, record_idx)) => {
                let tracking_node = &graph.matrix[*frame_idx][*record_idx];
                on_tracking_node(tracking_node);
                node = tracking_node.outs.iter().find(|o| o.0 == *id);
            }
            None => break,
        }
    }
}

fn median(values: &Vec<u32>) -> u32 {
    let len = values.len();
    if len % 2 == 0 {
        (values[len / 2 - 1] + values[len / 2]) / 2
    } else {
        values[len / 2]
    }
}

/// Computes the chain length metrics of a tracking graph.
pub fn eval_tracking_chain_length(graph: &api::TrackingGraph) -> api::EvalMetricChainLength {
    let mut lengths: Vec<u32> = Vec::new();
    for (id, _, _) in graph.root.outs.iter() {
        let mut length = 0;
        let mut on_tracking_node = |_: &api::TrackingNode| {
            length += 1;
        };
        follow_chain(&graph, *id, &mut on_tracking_node);
        lengths.push(length);
    }

    lengths.sort_unstable();

    if lengths.is_empty() {
        return api::EvalMetricChainLength {
            average: 0.0,
            median: 0.0,
            max: 0.0,
            min: 0.0,
        };
    }

    api::EvalMetricChainLength {
        average: lengths.iter().sum::<u32>() as f32 / lengths.len() as f32,
        median: median(&lengths) as f32,
        max: lengths[lengths.len() - 1] as f32,
        min: lengths[0] as f32,
    }
}

/// Computes the graph properties of a tracking graph.
pub fn eval_tracking_graph_properties(
    graph: &api::TrackingGraph,
) -> api::EvalMetricGraphProperties {
    let mut properties = api::EvalMetricGraphProperties {
        match_ratios: Vec::new(),
        conflict_ratios: Vec::new(),
    };
    for frame in graph.matrix.iter() {
        let mut n_matchs = 0;
        let mut n_conflicts = 0;
        for node in frame.iter() {
            if node.ins.len() > 0 {
                n_matchs += 1;
            }
            if node.ins.len() > 1 {
                n_conflicts += 1;
            }
        }
        properties
            .match_ratios
            .push(n_matchs as f32 / frame.len() as f32);
        properties
            .conflict_ratios
            .push(n_conflicts as f32 / frame.len() as f32);
    }

    properties
}
