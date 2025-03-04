use std::collections::HashSet;

use crate::{api, id::ID};

/// Follows a chain of tracking nodes and calls the given on_tracking_node function for each node.
fn follow_chain(
    graph: &api::TrackingGraph,
    id: ID,
    on_tracking_node: &mut dyn FnMut(&api::GraphNode),
) {
    let mut node = graph.root.outs.iter().find(|o| o.0 == id);

    loop {
        match node {
            Some((id, ch)) => {
                let tracking_node = &graph.matrix[ch.frame_idx][ch.record_idx];
                on_tracking_node(tracking_node);
                node = tracking_node.outs.iter().find(|(i, _)| *i == *id);
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
    for (id, _) in graph.root.outs.iter() {
        let mut length = 0;
        let mut on_tracking_node = |_: &api::GraphNode| {
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
            histogram: Vec::new(),
        };
    }

    let max = lengths[lengths.len() - 1];

    // build histogram with count of chain with certain length
    let mut histogram = (0..max + 1).map(|_| 0).collect::<Vec<u32>>();
    for length in lengths.iter() {
        histogram[*length as usize] += 1;
    }

    api::EvalMetricChainLength {
        average: lengths.iter().sum::<u32>() as f32 / lengths.len() as f32,
        median: median(&lengths) as f32,
        max: max as f32,
        min: lengths[0] as f32,
        histogram,
    }
}

/// Computes the graph properties of a tracking graph.
pub fn eval_tracking_graph_properties(
    graph: &api::TrackingGraph,
) -> api::EvalMetricGraphProperties {
    let mut properties = api::EvalMetricGraphProperties {
        records_match_ratios: Vec::new(),
        trackers_match_ratios: Vec::new(),
        conflict_ratios: Vec::new(),
    };
    for frame in graph.matrix.iter() {
        let mut n_record_matchs = 0;
        let mut n_conflicts = 0;
        let mut n_tracker_matchs = 0;
        let mut tracker_ids = HashSet::new();
        for node in frame.iter() {
            if node.ins.len() > 0 {
                n_record_matchs += 1;
            }
            if node.ins.len() > 1 {
                n_conflicts += 1;
            }

            for (id, _) in node.outs.iter() {
                tracker_ids.insert(id);
            }

            let in_trackers: HashSet<ID> = node.ins.iter().map(|(id, _)| *id).collect();
            for (id, _) in node.outs.iter() {
                if in_trackers.contains(id) {
                    n_tracker_matchs += 1;
                }
            }
        }

        properties
            .records_match_ratios
            .push(n_record_matchs as f32 / frame.len() as f32);
        properties
            .trackers_match_ratios
            .push(n_tracker_matchs as f32 / tracker_ids.len() as f32);
        properties
            .conflict_ratios
            .push(n_conflicts as f32 / frame.len() as f32);
    }

    properties
}
