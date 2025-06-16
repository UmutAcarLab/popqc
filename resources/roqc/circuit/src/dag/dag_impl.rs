use std::collections::HashSet;

use petgraph::{
    stable_graph::{NodeIndex, StableGraph},
    visit::EdgeRef,
};

use crate::types::{GateIndex, QubitIndex};
use crate::Gate;

#[derive(Debug, Clone)]
pub struct GateNode {
    pub gate: Gate,
    pub vector_clock: Vec<GateIndex>, //vector clock for each qubit, stores the node index of the last gate that acted on the qubit
}
#[derive(Debug, Clone)]
pub struct DAG {
    // A wrapper around a petgraph::StableGraph, to make sure that node indexes never get reused.
    graph: StableGraph<GateNode, QubitIndex, petgraph::Directed>,
    current_gate_index: GateIndex, // monotonic increasing counter for gate indexes
    pub gate_to_node: std::collections::HashMap<GateIndex, NodeIndex>,
    node_to_gate: std::collections::HashMap<NodeIndex, GateIndex>,
    unoptimized_gates: HashSet<GateIndex>,
}
impl Default for DAG {
    fn default() -> Self {
        Self::new()
    }
}

impl DAG {
    pub fn new() -> Self {
        Self {
            graph: StableGraph::new(),
            current_gate_index: 0,
            gate_to_node: std::collections::HashMap::new(),
            node_to_gate: std::collections::HashMap::new(),
            unoptimized_gates: HashSet::new(),
        }
    }
    pub fn add_gate(&mut self, gate: GateNode) -> GateIndex {
        let node = self.graph.add_node(gate);
        let gate_index = self.current_gate_index;
        self.current_gate_index += 1;
        self.gate_to_node.insert(gate_index, node);
        self.node_to_gate.insert(node, gate_index);
        if gate_index != 0 && gate_index != 1 {
            self.unoptimized_gates.insert(gate_index);
        }
        gate_index
    }
    pub fn remove_gate(&mut self, gate_index: GateIndex) {
        let node = self.gate_to_node.remove(&gate_index).unwrap();
        self.node_to_gate.remove(&node);
        self.gate_to_node.remove(&gate_index);
        self.graph.remove_node(node);
        self.unoptimized_gates.remove(&gate_index);
    }
    pub fn invalidate_neighbors(&mut self, gate_indices: HashSet<GateIndex>, omega: usize) {
        let mut frontier = gate_indices.clone();
        let mut visited = gate_indices.clone();
        for _ in 0..omega {
            let new_neighbors: Vec<_> = frontier
                .iter()
                .flat_map(|&gate_index| self.graph.neighbors(self.gate_to_node[&gate_index]))
                .collect();
            frontier.clear();
            new_neighbors.iter().for_each(|node| {
                let gate_index = self.node_to_gate[node];
                if gate_index != 0 && gate_index != 1 {
                    // Do not invalidate the start and end gates
                    if !visited.contains(&gate_index) {
                        visited.insert(gate_index);
                        frontier.insert(gate_index);
                        self.unoptimized_gates.insert(gate_index);
                    }
                }
            });
        }
    }
    pub fn get_neighbors(&self, gate_index: GateIndex, steps: usize) -> Vec<GateIndex> {
        let mut frontier = HashSet::new();
        let mut visited = HashSet::new();
        frontier.insert(gate_index);
        visited.insert(gate_index);
        for _ in 0..steps {
            let new_neighbors: Vec<_> = frontier
                .iter()
                .flat_map(|&gate_index| {
                    self.graph
                        .neighbors_undirected(self.gate_to_node[&gate_index])
                })
                .filter(|&node| !visited.contains(&self.node_to_gate[&node]))
                .filter(|&node| (self.node_to_gate[&node] != 0) && (self.node_to_gate[&node] != 1))
                .collect();
            frontier.clear();
            new_neighbors.iter().for_each(|node| {
                let gate_index = self.node_to_gate[node];
                visited.insert(gate_index);
                frontier.insert(gate_index);
            });
        }
        visited.into_iter().collect()
    }
    pub fn next_unoptimized_gate(&mut self) -> Option<GateIndex> {
        //not very parallelizable, but it's fine for now
        self.unoptimized_gates.iter().next().cloned()
    }

    pub fn set_optimized(&mut self, gate_index: GateIndex) {
        self.unoptimized_gates.remove(&gate_index);
    }

    pub fn get_gate(&self, gate_index: GateIndex) -> &GateNode {
        &self.graph[self.gate_to_node[&gate_index]]
    }
    pub fn get_gate_mut(&mut self, gate_index: GateIndex) -> &mut GateNode {
        &mut self.graph[self.gate_to_node[&gate_index]]
    }
    pub fn add_edge(&mut self, source: GateIndex, target: GateIndex, qubit: QubitIndex) {
        self.graph.add_edge(
            self.gate_to_node[&source],
            self.gate_to_node[&target],
            qubit,
        );
    }
    pub fn succ_neighbors(&self, gate_index: GateIndex) -> Vec<(QubitIndex, GateIndex)> {
        let edges = self.graph.edges_directed(
            self.gate_to_node[&gate_index],
            petgraph::Direction::Outgoing,
        );
        let mut res = Vec::new();
        for edge in edges {
            let qubit = *edge.weight();
            let gate_index = self.node_to_gate[&edge.target()];
            res.push((qubit, gate_index));
        }
        res
    }
    pub fn pred_neighbors(&self, gate_index: GateIndex) -> Vec<(QubitIndex, GateIndex)> {
        let edges = self.graph.edges_directed(
            self.gate_to_node[&gate_index],
            petgraph::Direction::Incoming,
        );
        let mut res = Vec::new();
        for edge in edges {
            let qubit = *edge.weight();
            let gate_index = self.node_to_gate[&edge.source()];
            res.push((qubit, gate_index));
        }
        res
    }
    pub fn succ_neighbor_qubit(&self, gate_index: GateIndex, qubit: QubitIndex) -> GateIndex {
        let edges = self.graph.edges_directed(
            self.gate_to_node[&gate_index],
            petgraph::Direction::Outgoing,
        );
        for edge in edges {
            if qubit == *edge.weight() {
                return self.node_to_gate[&edge.target()];
            }
        }
        panic!("No such edge");
    }
    pub fn pred_neighbor_qubit(&self, gate_index: GateIndex, qubit: QubitIndex) -> GateIndex {
        let edges = self.graph.edges_directed(
            self.gate_to_node[&gate_index],
            petgraph::Direction::Incoming,
        );
        for edge in edges {
            if qubit == *edge.weight() {
                return self.node_to_gate[&edge.source()];
            }
        }
        panic!("No such edge");
    }
    pub fn remove_edge(&mut self, source: GateIndex, target: GateIndex, qubit: QubitIndex) {
        let edges = self
            .graph
            .edges_connecting(self.gate_to_node[&source], self.gate_to_node[&target]);
        for edge in edges {
            if *edge.weight() == qubit {
                self.graph.remove_edge(edge.id());
                return;
            }
        }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
    pub fn node_weights(&self) -> impl Iterator<Item = &GateNode> {
        self.graph.node_weights()
    }
    pub fn toposort(&self) -> Vec<GateIndex> {
        petgraph::algo::toposort(&self.graph, None)
            .unwrap()
            .iter()
            .map(|node| self.node_to_gate[node])
            .collect()
    }
    pub fn contains_edge(&self, source: GateIndex, target: GateIndex) -> bool {
        self.graph
            .contains_edge(self.gate_to_node[&source], self.gate_to_node[&target])
    }
    pub fn to_gate_vec(&self) -> Vec<Gate> {
        let mut gates = vec![];
        let topo_order = self.toposort();
        for gate_idx in topo_order {
            let gate = self.get_gate(gate_idx).gate.clone();
            if gate != Gate::B {
                gates.push(gate);
            }
        }
        gates
    }
}
