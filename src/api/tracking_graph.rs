use pyo3::{exceptions::PyValueError, pyclass, pymethods, types::PyBytes, Bound, PyResult, Python};
use serde::{Deserialize, Serialize};

use crate::{
    frame::Frame,
    id::ID,
    trackers::{self, TrackingChain},
};

/// TrackingNode
///
/// Node in the tracking graph.
#[pyclass(frozen)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackingNode {
    /// List of incoming edges.
    ///
    /// Each edge represents a link from a previous chain node in a tracking chain.
    /// Each tuple contains the ID of the chain, the frame index and record index of the previous chain node.
    #[pyo3(get)]
    pub ins: Vec<(ID, usize, usize)>,
    /// List of outgoing edges.
    ///
    /// Each edge represents a link to a next chain node in a tracking chain.
    /// Each tuple contains the ID of the chain, the frame index and record index of the next chain node.
    #[pyo3(get)]
    pub outs: Vec<(ID, usize, usize)>,
}

#[pymethods]
impl TrackingNode {}

impl TrackingNode {
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
    pub root: TrackingNode,
    /// Adjacency matrix of the graph.
    ///
    /// Each row represents a frame, each column represents a record in the frame.
    #[pyo3(get)]
    pub matrix: Vec<Vec<TrackingNode>>,
}

#[pymethods]
impl TrackingGraph {
    /// Deserialize a tracking graph from bytes.
    #[staticmethod]
    pub fn from_bytes(bytes: &[u8]) -> PyResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|_| PyValueError::new_err("failed to deserialize TrackingGraph"))
    }

    /// Serialize the tracking graph to bytes.
    pub fn to_bytes<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyBytes>> {
        let bytes = bincode::serialize(self)
            .map_err(|_| PyValueError::new_err("failed to serialize TrackingGraph"))?;
        Ok(PyBytes::new_bound(py, &bytes))
    }
}

impl TrackingGraph {
    /// Create a new tracking graph from a list of frames and tracking chains.
    pub fn from_tracking_chains(frames: &Vec<Frame>, chains: Vec<TrackingChain>) -> Self {
        let mut matrix: Vec<Vec<TrackingNode>> = Vec::with_capacity(frames.len());
        for frame in frames.iter() {
            let mut column = Vec::with_capacity(frame.num_records());
            for _ in 0..frame.num_records() {
                column.push(TrackingNode::new());
            }
            matrix.push(column);
        }

        let mut root = TrackingNode::new();

        for chain in chains.iter() {
            let mut prev_node = &mut root;
            let mut prev_cn: Option<&trackers::TrackingNode> = None;

            for cn in chain.nodes.iter() {
                prev_node.outs.push((chain.id, cn.frame_idx, cn.record_idx));
                let next_node = &mut matrix[cn.frame_idx][cn.record_idx];

                if let Some(prev_cn) = prev_cn {
                    next_node
                        .ins
                        .push((chain.id, prev_cn.frame_idx, prev_cn.record_idx));
                }

                prev_node = next_node;
                prev_cn = Some(cn);
            }
        }

        Self { root, matrix }
    }

    /// Builds a tracking chain from the graph.
    pub fn build_tracking_chain(&self, id: ID) -> TrackingChain {
        let mut nodes = Vec::new();
        let mut node = self.root.outs.iter().find(|o| o.0 == id);

        loop {
            match node {
                Some((id, frame_idx, record_idx)) => {
                    let tracking_node = &self.matrix[*frame_idx][*record_idx];
                    nodes.push(trackers::TrackingNode::new(*frame_idx, *record_idx));
                    node = tracking_node.outs.iter().find(|o| o.0 == *id);
                }
                None => break,
            }
        }

        TrackingChain::new(id, nodes)
    }
}
