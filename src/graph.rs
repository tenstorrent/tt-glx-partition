// SPDX-FileCopyrightText: © 2025 Tenstorrent Inc.
// SPDX-License-Identifier: Apache-2.0

//! Galaxy graph primitives and helpers.
//!
//! This module defines the logical topology (`GraphCoords`, `Edges`) and provides helpers to:
//! - Build the full galaxy connectivity graph (`generate_galaxy_graph`)
//! - Cache fast adjacency lookups (`CachedGraph`)
//! - Compute which ports must be disabled to isolate a connected subset (`get_edges_to_disable`)
use crate::constants::UBB_GRAPH_EDGES;
use petgraph::graph::{NodeIndex, UnGraph};
use std::collections::{HashMap, HashSet};

/// Node identifier expressed as 2D coordinates:
/// - `0`: UBB index (0-based to 3)
/// - `1`: ASIC index within that UBB (0-based to 7)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GraphCoords(pub usize, pub usize);
impl std::fmt::Display for GraphCoords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x:{}; y:{}", self.0, self.1)
    }
}

/// Undirected graph edge describing the connected endpoints and the port on each endpoint.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Edges {
    pub node_1: GraphCoords,
    pub node_2: GraphCoords,
    pub node_1_port: &'static str,
    pub node_2_port: &'static str,
}

/// Per-node port disables needed for isolation.
///
/// `ports` contains the vec of port ranges on `node` that should be
/// disabled.
#[derive(Debug, Clone)]
pub struct EdgeToDisable {
    pub node: GraphCoords,
    pub ports: Vec<&'static str>,
}

/// Union Find structure to check connectivity between node subsets
/// Implementation example followed - <https://brilliant.org/wiki/disjoint-set-data-structure/>
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
        }
    }

    /// Find a node with path compression
    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    /// Union by rank and update the parent pointer
    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x != root_y {
            if self.rank[root_x] < self.rank[root_y] {
                self.parent[root_x] = root_y;
            } else if self.rank[root_x] > self.rank[root_y] {
                self.parent[root_y] = root_x;
            } else {
                self.parent[root_y] = root_x;
                self.rank[root_x] += 1;
            }
        }
    }
}

/// Builds the full 32-node galaxy connectivity graph using [`UBB_GRAPH_EDGES`].
pub fn generate_galaxy_graph() -> UnGraph<GraphCoords, Edges> {
    // Generate a petgraph undirected graph representing the UBB connections.
    let mut graph = UnGraph::<GraphCoords, Edges>::new_undirected();

    for i in 0..4 {
        for j in 0..8 {
            graph.add_node(GraphCoords(i, j));
        }
    }
    for edge in UBB_GRAPH_EDGES.iter() {
        // get node index for each edge
        let node_a = graph
            .node_indices()
            .find(|&idx| graph[idx] == edge.node_1)
            .unwrap();
        let node_b = graph
            .node_indices()
            .find(|&idx| graph[idx] == edge.node_2)
            .unwrap();
        graph.add_edge(node_a, node_b, edge.to_owned());
    }

    graph
}

/// Cached view of the galaxy graph for fast connectivity / neighbor queries.
#[derive(Debug, Clone)]
pub struct CachedGraph {
    /// Underlying petgraph structure (needed for edge/port lookups).
    pub graph: UnGraph<GraphCoords, Edges>,
    /// Bitmap adjacency list: `adjacency_bitmask[i]` has bit `j` set if node `i` connects to `j`.
    pub adjacency_bitmask: [u32; 32],
    /// Mapping from `GraphCoords` to petgraph `NodeIndex`.
    pub node_to_index: HashMap<GraphCoords, NodeIndex>,
}

impl CachedGraph {
    /// Creates a new [`CachedGraph`] by building an adjacency bitmask and reverse index mapping.
    pub fn new(graph: UnGraph<GraphCoords, Edges>) -> Self {
        let mut adjacency_bitmask: [u32; 32] = [0; 32];
        let mut node_to_index = HashMap::new();

        // Validate we have exactly 32 nodes
        assert_eq!(
            graph.node_count(),
            32,
            "Graph must have exactly 32 nodes - some devices are missing from pcie detection."
        );

        // Build adjacency bitmask and mappings
        for node_index in graph.node_indices() {
            let graph_coords = graph[node_index];
            let idx = node_index.index();
            node_to_index.insert(graph_coords, node_index);

            for neighbor in graph.neighbors(node_index) {
                let neighbor_idx = neighbor.index();
                adjacency_bitmask[idx] |= 1 << neighbor_idx;
            }
        }
        CachedGraph {
            graph,
            adjacency_bitmask,
            node_to_index,
        }
    }

    /// Returns true if `selected_coords` forms a single connected component in the graph.
    pub fn is_subset_connected_union_find(&self, selected_coords: &[GraphCoords]) -> bool {
        if selected_coords.len() <= 1 {
            return true;
        }

        // Convert to indices
        let indices: Vec<usize> = selected_coords
            .iter()
            .filter_map(|c| self.node_to_index.get(c).map(|n| n.index()))
            .collect();
        if indices.len() != selected_coords.len() {
            return false;
        }

        // Create Union-Find structure
        let mut uf = UnionFind::new(32);
        let index_set: HashSet<usize> = indices.iter().copied().collect();

        // Union all edges between selected nodes
        for &idx in &indices {
            let neighbors_mask = self.adjacency_bitmask[idx];

            // Check each neighbor
            for neighbor_idx in 0..32 {
                if (neighbors_mask >> neighbor_idx) & 1 == 1 && index_set.contains(&neighbor_idx) {
                    // if neighbor is in selected set, then union them
                    uf.union(idx, neighbor_idx);
                }
            }
        }

        // Check if all selected nodes are in the same component
        let first_root = uf.find(indices[0]);
        indices.iter().all(|&idx| uf.find(idx) == first_root)
    }

    /// For a connected subset, returns all ports that must be disabled to isolate it.
    ///
    /// For each selected node, we examine all adjacent neighbors. If a neighbor is outside the
    /// selected set, we record the local port on the selected node that connects to that neighbor
    /// and mark it for disabling.
    pub fn get_edges_to_disable(&self, selected_coords: &[GraphCoords]) -> Vec<EdgeToDisable> {
        // Create a HashSet for fast membership checking
        let selected_set: HashSet<GraphCoords> = selected_coords.iter().copied().collect();
        let mut edges_to_disable: Vec<EdgeToDisable> = Vec::new();

        // For each selected coordinate, use adjacency bitmask to find neighbors
        for &coords in selected_coords {
            // Get the node index for this coordinate
            let node_index = match self.node_to_index.get(&coords) {
                Some(&idx) => idx,
                None => continue, // Skip if node not found
            };

            let current_idx = node_index.index();
            let neighbors_mask = self.adjacency_bitmask[current_idx];

            // Iterate through neighbors using the adjacency bitmask
            for neighbor_idx in 0..32 {
                // Check if this neighbor is connected (bit is set)
                if (neighbors_mask >> neighbor_idx) & 1 == 1 {
                    // Convert neighbor index back to GraphCoords
                    let neighbor_node_index = NodeIndex::new(neighbor_idx);
                    let neighbor_coords = self.graph[neighbor_node_index];

                    // If the neighbor is NOT in the selected set, this edge needs to be disabled
                    if !selected_set.contains(&neighbor_coords) {
                        // Find the edge between current node and neighbor to get port info
                        if let Some(edge_idx) =
                            self.graph.find_edge(node_index, neighbor_node_index)
                        {
                            let edge_data = &self.graph[edge_idx];

                            // Determine which port to disable based on which node is which
                            let port_to_disable = if edge_data.node_1 == coords {
                                edge_data.node_1_port
                            } else {
                                edge_data.node_2_port
                            };

                            // Check if node exists in edges_to_disable, if not add it with port to disable
                            if !edges_to_disable.iter().any(|e| e.node == coords) {
                                edges_to_disable.push(EdgeToDisable {
                                    node: coords,
                                    ports: vec![port_to_disable],
                                });
                            } else {
                                edges_to_disable
                                    .iter_mut()
                                    .find(|e| e.node == coords)
                                    .unwrap()
                                    .ports
                                    .push(port_to_disable);
                            }
                        }
                    }
                }
            }
        }

        edges_to_disable
    }
}
