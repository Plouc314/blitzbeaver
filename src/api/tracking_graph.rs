use pyo3::{pyclass, pymethods};
use serde::{Deserialize, Serialize};

use crate::{frame::Frame, id::ID, trackers::TrackingChain};

/// ChainNode
///
/// References a record in a frame.
///
/// Note: this doesn't hold the record itself, but only the indices to access it.
#[pyclass(frozen)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ChainNode {
    #[pyo3(get)]
    pub frame_idx: usize,
    #[pyo3(get)]
    pub record_idx: usize,
}

impl ChainNode {
    pub fn new(frame_idx: usize, record_idx: usize) -> Self {
        Self {
            frame_idx,
            record_idx,
        }
    }
}

/// GraphNode
///
/// Node in the tracking graph.
#[pyclass(frozen)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphNode {
    /// List of incoming edges.
    ///
    /// Each edge represents a link from a previous chain node in a tracking chain.
    /// Each tuple contains the ID of the chain and the chain node.
    #[pyo3(get)]
    pub ins: Vec<(ID, ChainNode)>,
    /// List of outgoing edges.
    ///
    /// Each edge represents a link to a next chain node in a tracking chain.
    /// Each tuple contains the ID of the chain and the chain node.
    #[pyo3(get)]
    pub outs: Vec<(ID, ChainNode)>,
}

#[pymethods]
impl GraphNode {}

impl GraphNode {
    pub fn new() -> Self {
        Self {
            ins: Vec::new(),
            outs: Vec::new(),
        }
    }
}

/// TrackingGraph
///
/// Graph representing all tracking chains, each node in the graph
/// represents a record in a frame. Each edge represents a link between
/// two chain nodes of a tracking chain.
#[pyclass(frozen)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackingGraph {
    /// Source of the graph, all tracking chains start from this node.
    #[pyo3(get)]
    pub root: GraphNode,
    /// Adjacency matrix of the graph.
    ///
    /// Each row represents a frame, each column represents a record in the frame.
    #[pyo3(get)]
    pub matrix: Vec<Vec<GraphNode>>,
}

#[pymethods]
impl TrackingGraph {
    /// Python function
    ///
    /// Builds the tracking chain with the given ID.
    pub fn get_tracking_chain(&self, id: ID) -> Vec<ChainNode> {
        self.build_chain_node_vector(id)
    }
}

impl TrackingGraph {
    /// Create a new tracking graph from a list of frames and tracking chains.
    pub fn from_tracking_chains(frames: &Vec<Frame>, chains: Vec<TrackingChain>) -> Self {
        let mut matrix: Vec<Vec<GraphNode>> = Vec::with_capacity(frames.len());
        for frame in frames.iter() {
            let mut column = Vec::with_capacity(frame.num_records());
            for _ in 0..frame.num_records() {
                column.push(GraphNode::new());
            }
            matrix.push(column);
        }

        let mut root = GraphNode::new();

        for chain in chains.iter() {
            let mut prev_node = &mut root;
            let mut prev_cn: Option<&ChainNode> = None;

            for cn in chain.nodes.iter() {
                prev_node.outs.push((chain.id, cn.clone()));
                let next_node = &mut matrix[cn.frame_idx][cn.record_idx];

                if let Some(prev_cn) = prev_cn {
                    next_node.ins.push((chain.id, prev_cn.clone()));
                }

                prev_node = next_node;
                prev_cn = Some(cn);
            }
        }

        Self { root, matrix }
    }

    /// Builds the tracking chain with the given ID.
    pub fn build_tracking_chain(&self, id: ID) -> TrackingChain {
        let nodes = self.build_chain_node_vector(id);
        TrackingChain::new(id, nodes)
    }

    /// Builds a vector of chain nodes composing a tracking chain.
    fn build_chain_node_vector(&self, id: ID) -> Vec<ChainNode> {
        let mut node = self.root.outs.iter().find(|o| o.0 == id);

        let mut nodes = Vec::new();

        loop {
            match node {
                Some((id, ch)) => {
                    let tracking_node = &self.matrix[ch.frame_idx][ch.record_idx];
                    nodes.push(ch.clone());
                    node = tracking_node.outs.iter().find(|o| o.0 == *id);
                }
                None => break,
            }
        }

        nodes
    }
}
