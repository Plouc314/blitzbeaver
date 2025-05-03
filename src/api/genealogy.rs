use std::collections::HashMap;

use pyo3::{pyclass, pyfunction, pymethods, PyResult};
use pyo3_polars::PyDataFrame;
use serde::Serialize;

use crate::{
    distances::{DistanceMetric, LvDistanceMetric},
    frame::Frame,
    id::ID,
    trackers::TrackingChain,
    word::Word,
};

use super::{casting, ChainNode, DistanceMetricConfig, RecordSchema, TrackingGraph};

/// Genealogy node using IDs for indirect references.
///
/// This ID used is the ID of the tracking chain in the tracking graph.
pub struct IndirectGenealogyNode {
    pub id: ID,
    pub parent_id: Option<ID>,
    pub is_husband: bool,
    pub children: Vec<ID>,
    /// Leaf children are children whose name appears only in the children column.
    /// But could not be matched with an existing tracking chain.
    pub leaf_children: Vec<String>,
}

/// Compact representation of a genealogy node.
///
/// This is used for serialization to JSON.
#[derive(Debug, Serialize)]
pub struct CompactGenealogyNode {
    pub id: ID,
    pub is_husband: bool,
    pub children: Vec<CompactGenealogyNode>,
    /// Leaf children are children whose name appears only in the children column.
    /// But could not be matched with an existing tracking chain.
    pub leaf_children: Vec<String>,
}

/// Child edge between a parent and a child.
///
/// This represents a relationship between a parent and a child.
/// The `is_husband` field indicates if the child is a husband or a wife.
pub struct ChildEdge {
    pub parent_id: ID,
    pub child_id: ID,
    pub is_husband: bool,
}

/// Child record found in the children column.
///
/// This is used to keep track of the number of occurrences of a child name
/// and the last frame index where it was found.
struct ChildRecord {
    name: Word,
    count: usize,
    last_frame_idx: usize,
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct GenealogyConfig {
    #[pyo3(get)]
    pub husband_name_idx: usize,
    #[pyo3(get)]
    pub wife_name_idx: usize,
    #[pyo3(get)]
    pub last_name_idx: usize,
    #[pyo3(get)]
    pub origin_idx: usize,
    #[pyo3(get)]
    pub children_idx: usize,
    #[pyo3(get)]
    pub min_tracking_chain_length: usize,
    #[pyo3(get)]
    pub min_child_count: usize,
    #[pyo3(get)]
    pub search_last_frame_child: bool,
    #[pyo3(get)]
    pub search_wife: bool,
    #[pyo3(get)]
    pub search_year_range: usize,
    #[pyo3(get)]
    pub matching_threshold: f32,
}

#[pymethods]
impl GenealogyConfig {
    #[new]
    pub fn py_new(
        husband_name_idx: usize,
        wife_name_idx: usize,
        last_name_idx: usize,
        origin_idx: usize,
        children_idx: usize,
        min_tracking_chain_length: usize,
        min_child_count: usize,
        search_last_frame_child: bool,
        search_wife: bool,
        search_year_range: usize,
        matching_threshold: f32,
    ) -> Self {
        Self {
            husband_name_idx,
            wife_name_idx,
            last_name_idx,
            origin_idx,
            children_idx,
            min_tracking_chain_length,
            min_child_count,
            search_last_frame_child,
            search_wife,
            search_year_range,
            matching_threshold,
        }
    }
}

pub struct GenealogyEngine {
    frames: Vec<Frame>,
    tracking_graph: TrackingGraph,
    config: GenealogyConfig,
    distance_metric: Box<dyn DistanceMetric<Word>>,
}

impl GenealogyEngine {
    pub fn new(
        frames: Vec<Frame>,
        tracking_graph: TrackingGraph,
        config: GenealogyConfig,
        distance_metric: Box<dyn DistanceMetric<Word>>,
    ) -> Self {
        Self {
            frames,
            tracking_graph,
            config,
            distance_metric,
        }
    }

    fn get_husband_name_at(&self, chain_node: ChainNode) -> Option<&Word> {
        let frame = &self.frames[chain_node.frame_idx];
        let husband_name_column = frame.column(self.config.husband_name_idx);
        let husband_name = husband_name_column[chain_node.record_idx].as_word();
        husband_name
    }

    fn get_wife_name_at(&self, chain_node: ChainNode) -> Option<&Word> {
        let frame = &self.frames[chain_node.frame_idx];
        let wife_name_column = frame.column(self.config.wife_name_idx);
        let wife_name = wife_name_column[chain_node.record_idx].as_word();
        wife_name
    }

    fn get_last_name_at(&self, chain_node: ChainNode) -> Option<&Word> {
        let frame = &self.frames[chain_node.frame_idx];
        let last_name_column = frame.column(self.config.last_name_idx);
        let last_name = last_name_column[chain_node.record_idx].as_word();
        last_name
    }

    fn get_origin_at(&self, chain_node: ChainNode) -> Option<&Word> {
        let frame = &self.frames[chain_node.frame_idx];
        let origin_column = frame.column(self.config.origin_idx);
        let origin = origin_column[chain_node.record_idx].as_word();
        origin
    }

    fn get_children_at(&self, chain_node: ChainNode) -> &Vec<Word> {
        let frame = &self.frames[chain_node.frame_idx];
        let children_column = frame.column(self.config.children_idx);
        let children = children_column[chain_node.record_idx].as_multiword();
        children
    }

    fn get_child_records(&self, tracking_chain: &TrackingChain) -> Vec<ChildRecord> {
        let mut records: HashMap<&str, ChildRecord> = HashMap::new();

        for node in tracking_chain.nodes.iter() {
            let children = self.get_children_at(*node);
            for child in children.iter() {
                if let Some(record) = records.get_mut(child.raw.as_str()) {
                    record.count += 1;
                    record.last_frame_idx = node.frame_idx;
                } else {
                    records.insert(
                        &child.raw.as_str(),
                        ChildRecord {
                            name: child.clone(),
                            count: 1,
                            last_frame_idx: node.frame_idx,
                        },
                    );
                }
            }
        }

        records.into_values().collect()
    }

    /// Finds all tracking chains meeting the criteria.
    ///
    /// For each tracking chain, returns the child records and the uneligible children.
    ///
    /// The uneligible children are those that are not considered for matching but still
    /// reliably appear in the children column.
    fn find_tracking_chains_of_interest(
        &self,
    ) -> Vec<(TrackingChain, Vec<ChildRecord>, Vec<String>)> {
        let mut chains_of_interest = Vec::new();
        for (id, _) in self.tracking_graph.root.outs.iter() {
            let tracking_chain = self.tracking_graph.build_tracking_chain(*id);

            if tracking_chain.nodes.len() < self.config.min_tracking_chain_length {
                continue;
            }

            let mut child_records = Vec::new();
            let mut uneligible_children = Vec::new();
            for child_record in self.get_child_records(&tracking_chain).into_iter() {
                if child_record.count < self.config.min_child_count {
                    continue;
                }
                if !self.config.search_last_frame_child
                    && child_record.last_frame_idx == tracking_chain.nodes.last().unwrap().frame_idx
                {
                    uneligible_children.push(child_record.name.raw.clone());
                    continue;
                }
                child_records.push(child_record);
            }

            if child_records.len() == 0 {
                continue;
            }
            chains_of_interest.push((tracking_chain, child_records, uneligible_children));
        }
        chains_of_interest
    }

    /// Returns the the nodes of the tracking chains that start on the given frame.
    fn get_starting_nodes_at_frame(&self, frame_idx: usize) -> Vec<(ID, ChainNode)> {
        let mut starting_nodes = Vec::new();
        for node in self.tracking_graph.matrix[frame_idx].iter() {
            // search for chains starting on this frame
            // that is that have no incoming edge
            for (id, ch) in node.outs.iter() {
                if node.ins.iter().find(|(id2, _)| id2 == id).is_none() {
                    starting_nodes.push((*id, ch.clone()));
                }
            }
        }
        starting_nodes
    }

    /// Calculates the score of record for the given child.
    fn calculate_record_score(
        &mut self,
        chain_node: ChainNode,
        child_first_name: &Word,
        child_last_name: Option<&Word>,
        child_origin: Option<&Word>,
    ) -> (f32, bool) {
        // only consider the child if it has a last name and an origin
        let child_last_name = match child_last_name {
            Some(last_name) => last_name,
            None => return (0.0, false),
        };
        let child_origin = match child_origin {
            Some(origin) => origin,
            None => return (0.0, false),
        };

        // swap distance metric to avoid ownership issues
        let mut distance_metric = std::mem::replace(
            &mut self.distance_metric,
            Box::new(LvDistanceMetric::new(false)),
        );

        let husband_name = self.get_husband_name_at(chain_node);
        let wife_name = self.get_wife_name_at(chain_node);
        let last_name = self.get_last_name_at(chain_node);
        let origin = self.get_origin_at(chain_node);

        // there are two cases:
        // 1. the child becomes the husband:
        //    then the husband name and last name should match
        // 2. the child becomes the wife
        //    then the wife name should match

        let mut score_1 = 0.0;
        let mut score_2 = 0.0;

        // case 1
        match (husband_name, last_name, origin) {
            (Some(husband_name), Some(last_name), Some(origin)) => {
                score_1 = (distance_metric.dist(child_first_name, husband_name)
                    + distance_metric.dist(child_last_name, last_name)
                    + distance_metric.dist(child_origin, origin))
                    / 3.0;
            }
            _ => {}
        }

        // case 2
        if self.config.search_wife {
            match wife_name {
                Some(wife_name) => {
                    score_2 = distance_metric.dist(child_first_name, wife_name);
                }
                _ => {}
            }
        }

        // swap back the distance metric
        self.distance_metric = distance_metric;

        if score_1 > score_2 {
            (score_1, true)
        } else {
            (score_2, false)
        }
    }

    /// Searches for a tracking chain matching the child record.
    ///
    /// The search is done in the next `search_year_range` frames.
    fn search_child(
        &mut self,
        tracking_chain: &TrackingChain,
        child_record: &ChildRecord,
    ) -> Option<ChildEdge> {
        let chain_node = tracking_chain
            .nodes
            .iter()
            .find(|node| node.frame_idx == child_record.last_frame_idx)
            .unwrap();
        let child_first_name = child_record.name.clone();
        let child_last_name = self.get_last_name_at(*chain_node).cloned();
        let child_origin = self.get_origin_at(*chain_node).cloned();

        let mut edge = None;
        let mut edge_score = 0.0;

        let start_search_frame = child_record.last_frame_idx + 1;
        let end_search_frame = start_search_frame + self.config.search_year_range;

        for frame_idx in start_search_frame..end_search_frame {
            if frame_idx >= self.frames.len() {
                break;
            }
            for (id, node) in self.get_starting_nodes_at_frame(frame_idx) {
                let (score, is_husband) = self.calculate_record_score(
                    node,
                    &child_first_name,
                    child_last_name.as_ref(),
                    child_origin.as_ref(),
                );
                if score < self.config.matching_threshold {
                    continue;
                }
                if score > edge_score {
                    edge_score = score;
                    edge = Some(ChildEdge {
                        parent_id: tracking_chain.id,
                        child_id: id,
                        is_husband,
                    });
                }
            }
        }

        edge
    }

    /// Computes the child edges for all tracking chains of interest.
    ///
    /// Also returns the unmatched children for each tracking chain.
    fn compute_child_edges(&mut self) -> (Vec<ChildEdge>, HashMap<ID, Vec<String>>) {
        let mut edges = Vec::new();
        let mut unmatched_childrens = HashMap::new();
        let chains_of_interest = self.find_tracking_chains_of_interest();

        for (tracking_chain, child_records, uneligible_children) in chains_of_interest {
            let mut unmatched_children = uneligible_children;
            for child_record in child_records.iter() {
                match self.search_child(&tracking_chain, child_record) {
                    Some(edge) => {
                        edges.push(edge);
                    }
                    None => {
                        unmatched_children.push(child_record.name.raw.clone());
                    }
                }
            }
            if unmatched_children.len() > 0 {
                unmatched_childrens.insert(tracking_chain.id, unmatched_children);
            }
        }
        (edges, unmatched_childrens)
    }

    /// Computes the genealogy nodes map.
    ///
    /// This reconstructs genealogy trees using indirect references (IDs).
    fn compute_genealogy_nodes_map(&mut self) -> HashMap<ID, IndirectGenealogyNode> {
        let mut genealogy_nodes: HashMap<ID, IndirectGenealogyNode> = HashMap::new();
        let (child_edges, unmatched_childrens) = self.compute_child_edges();

        for edge in child_edges {
            if let Some(node) = genealogy_nodes.get_mut(&edge.parent_id) {
                node.children.push(edge.child_id);
            } else {
                genealogy_nodes.insert(
                    edge.parent_id,
                    IndirectGenealogyNode {
                        id: edge.parent_id,
                        parent_id: None,
                        is_husband: true,
                        children: vec![edge.child_id],
                        leaf_children: unmatched_childrens
                            .get(&edge.parent_id)
                            .unwrap_or(&Vec::new())
                            .clone(),
                    },
                );
            }
            if let Some(node) = genealogy_nodes.get_mut(&edge.child_id) {
                node.is_husband = edge.is_husband;
                node.parent_id = Some(edge.parent_id);
            } else {
                genealogy_nodes.insert(
                    edge.child_id,
                    IndirectGenealogyNode {
                        id: edge.child_id,
                        parent_id: Some(edge.parent_id),
                        is_husband: edge.is_husband,
                        children: Vec::new(),
                        leaf_children: unmatched_childrens
                            .get(&edge.child_id)
                            .unwrap_or(&Vec::new())
                            .clone(),
                    },
                );
            }
        }

        genealogy_nodes
    }

    /// Builds a compact genealogy node from the genealogy nodes map.
    fn build_compact_genealogy_node(
        &mut self,
        id: ID,
        genealogy_nodes_map: &HashMap<ID, IndirectGenealogyNode>,
    ) -> CompactGenealogyNode {
        match genealogy_nodes_map.get(&id) {
            Some(node) => CompactGenealogyNode {
                id: node.id,
                is_husband: node.is_husband,
                children: node
                    .children
                    .iter()
                    .map(|child_id| {
                        self.build_compact_genealogy_node(*child_id, genealogy_nodes_map)
                    })
                    .collect(),
                leaf_children: node.leaf_children.clone(),
            },
            None => panic!("Inconsistent genealogy tree: {} not found", id),
        }
    }

    /// Computes all genealogy trees in the tracking graph.
    pub fn compute_genealogy_trees(&mut self) -> Vec<CompactGenealogyNode> {
        let genealogy_nodes_map = self.compute_genealogy_nodes_map();
        let mut genealogy_trees = Vec::new();

        for node in genealogy_nodes_map.values() {
            if node.parent_id.is_none() {
                let compact_node = self.build_compact_genealogy_node(node.id, &genealogy_nodes_map);
                genealogy_trees.push(compact_node);
            }
        }

        genealogy_trees
    }
}

#[pyfunction]
pub fn execute_genealogy_process(
    genealogy_config: GenealogyConfig,
    distance_metric_config: &DistanceMetricConfig,
    record_schema: RecordSchema,
    tracking_graph: TrackingGraph,
    dataframes: Vec<PyDataFrame>,
) -> PyResult<String> {
    let mut frames = Vec::new();
    for i in 0..dataframes.len() {
        let frame = casting::cast_to_frame(i, &record_schema, &dataframes[i])?;
        frames.push(frame);
    }

    let distance_metric =
        casting::cast_distance_metric_config(distance_metric_config)?.make_metric();

    let mut genealogy_engine =
        GenealogyEngine::new(frames, tracking_graph, genealogy_config, distance_metric);

    let genealogy_trees = genealogy_engine.compute_genealogy_trees();
    let genealogy_json = serde_json::to_string(&genealogy_trees).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("JSON serialization error: {}", e))
    })?;
    Ok(genealogy_json)
}
