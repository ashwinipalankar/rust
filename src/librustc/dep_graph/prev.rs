use crate::ich::Fingerprint;
use rustc_data_structures::fx::FxHashMap;
use rustc_data_structures::indexed_vec::IndexVec;
use rustc_data_structures::sync::AtomicCell;
use super::dep_node::{DepNode, DepKind};
use super::graph::{DepNodeIndex, DepNodeState};
use super::serialized::SerializedDepGraph;

#[derive(Debug, Default)]
pub struct PreviousDepGraph {
    data: SerializedDepGraph,
    pub(super) index: FxHashMap<DepNode, DepNodeIndex>,
    pub(super) unused: Vec<DepNodeIndex>,
}

impl PreviousDepGraph {
    pub fn new_and_state(
        data: SerializedDepGraph
    ) -> (PreviousDepGraph, IndexVec<DepNodeIndex, AtomicCell<DepNodeState>>) {
        let mut unused = Vec::new();

        let state: IndexVec<_, _> = data.nodes.iter_enumerated().map(|(index, node)| {
            if node.kind == DepKind::Null {
                // There might be `DepKind::Null` nodes due to thread-local dep node indices
                // that didn't get assigned anything.
                // We also changed outdated nodes to `DepKind::Null`.
                unused.push(index);
                AtomicCell::new(DepNodeState::Invalid)
            } else {
                AtomicCell::new(DepNodeState::Unknown)
            }
        }).collect();

        let index: FxHashMap<_, _> = data.nodes
            .iter_enumerated()
            .filter_map(|(idx, &dep_node)| {
                if dep_node.kind == DepKind::Null {
                    None
                } else {
                    Some((dep_node, idx))
                }
            })
            .collect();

        (PreviousDepGraph { data, index, unused }, state)
    }

    #[inline]
    pub fn edge_targets_from(
        &self,
        dep_node_index: DepNodeIndex
    ) -> &[DepNodeIndex] {
        self.data.edge_targets_from(dep_node_index)
    }

    #[inline]
    pub fn index_to_node(&self, dep_node_index: DepNodeIndex) -> DepNode {
        self.data.nodes[dep_node_index]
    }

    #[inline]
    pub fn node_to_index(&self, dep_node: &DepNode) -> DepNodeIndex {
        self.index[dep_node]
    }

    #[inline]
    pub fn node_to_index_opt(&self, dep_node: &DepNode) -> Option<DepNodeIndex> {
        self.index.get(dep_node).cloned()
    }

    #[inline]
    pub fn fingerprint_of(&self, dep_node: &DepNode) -> Option<Fingerprint> {
        self.index
            .get(dep_node)
            .map(|&node_index| self.data.fingerprints[node_index])
    }

    #[inline]
    pub fn fingerprint_by_index(&self,
                                dep_node_index: DepNodeIndex)
                                -> Fingerprint {
        self.data.fingerprints[dep_node_index]
    }

    pub fn node_count(&self) -> usize {
        self.data.nodes.len()
    }
}
