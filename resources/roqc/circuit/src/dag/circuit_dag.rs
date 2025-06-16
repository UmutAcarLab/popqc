use crate::config::{Cost, Gateset};
use crate::seq::CircuitSeq;
use crate::types::{GateIndex, QubitIndex};
use crate::Gate;
use order_maintenance::Priority;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

pub use super::dag_impl::{GateNode, DAG};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    pub start: GateIndex,
    pub end: GateIndex,
    pub qubit: QubitIndex,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HalfEdge {
    pub node: GateIndex,
    pub qubit: QubitIndex,
}

pub struct CircuitDag {
    pub num_qubits: usize,
    pub graph: DAG,
    pub start_node: GateIndex,
    pub final_node: GateIndex,
    pub index_priority_map: HashMap<(GateIndex, QubitIndex), Priority>, //used to store the priority of each qubit in each node, need to be updated when calling insert_at (actually performed in get_new_vector_clock), need to be deleted when calling delete_at(not implemented yet)
}

impl CircuitDag {
    pub fn new_from_seq(circ: CircuitSeq) -> Self {
        CircuitDag::new(circ.gates, circ.num_qubits)
    }

    pub fn new(gates: Vec<Gate>, num_qubits: usize) -> Self {
        let mut graph = DAG::new();

        let start_node = graph.add_gate(GateNode {
            gate: Gate::B,
            vector_clock: Vec::new(),
        });
        graph.get_gate_mut(start_node).vector_clock = vec![start_node; num_qubits];

        let final_node = graph.add_gate(GateNode {
            gate: Gate::B,
            vector_clock: Vec::new(),
        });
        graph.get_gate_mut(final_node).vector_clock = vec![final_node; num_qubits];
        for q in 0..num_qubits {
            graph.add_edge(start_node, final_node, q);
        }
        let mut index_priority_map = HashMap::new();
        for q in 0..num_qubits {
            let this_priority = Priority::new();
            index_priority_map.insert((start_node, q), this_priority.clone());
            index_priority_map.insert((final_node, q), this_priority.insert());
        }
        let mut this_dag = Self {
            num_qubits,
            graph,
            start_node,
            final_node,
            index_priority_map,
        };
        let mut frontier = vec![0; num_qubits];
        for gate in &gates {
            let indices = gate.qubits().iter().map(|q| (*q, frontier[*q])).collect();
            let new_index = this_dag.insert_at(indices, gate.clone());
            gate.qubits().iter().for_each(|q| {
                frontier[*q] = new_index;
            });
        }
        this_dag
    }
    fn get_priority(&mut self, gate_idx: GateIndex, qubit: QubitIndex) -> Priority {
        let vector_clock = self.graph.get_gate(gate_idx).vector_clock[qubit];
        self.index_priority_map
            .get(&(vector_clock, qubit))
            .unwrap()
            .clone()
    }
    fn get_new_vector_clock(&mut self, idx: GateIndex) -> Vec<GateIndex> {
        // also responsible to update index_priority_map
        let prev_indices = self
            .graph
            .pred_neighbors(idx)
            .iter()
            .map(|(_, p)| *p)
            .collect::<Vec<_>>();
        let mut new_vector_clock = vec![0; self.num_qubits];
        for (q, clock_at_q) in new_vector_clock
            .iter_mut()
            .enumerate()
            .take(self.num_qubits)
        {
            let mut latest_idx = prev_indices[0];
            for p in prev_indices.iter() {
                if self.get_priority(*p, q) > self.get_priority(latest_idx, q) {
                    latest_idx = *p;
                }
            }
            *clock_at_q = self.graph.get_gate(latest_idx).vector_clock[q];
        }
        for q in self.graph.get_gate(idx).gate.qubits() {
            let this_priority = self.get_priority(new_vector_clock[q], q);
            self.index_priority_map
                .insert((idx, q), this_priority.insert());
            new_vector_clock[q] = idx;
        }
        new_vector_clock
    }
    fn update_vector_clock(&mut self, idx: GateIndex) -> Vec<GateIndex> {
        if idx == self.start_node || idx == self.final_node {
            return self.graph.get_gate(idx).vector_clock.clone();
        }
        let prev_indices = self
            .graph
            .pred_neighbors(idx)
            .iter()
            .map(|(_, p)| *p)
            .collect::<Vec<_>>();
        let mut new_vector_clock = vec![0; self.num_qubits];
        for (q, item) in new_vector_clock
            .iter_mut()
            .enumerate()
            .take(self.num_qubits)
        {
            let mut latest_idx = prev_indices[0];
            for p in prev_indices.iter() {
                if self.get_priority(*p, q) > self.get_priority(latest_idx, q) {
                    latest_idx = *p;
                }
            }
            *item = self.graph.get_gate(latest_idx).vector_clock[q];
        }
        for q in self.graph.get_gate(idx).gate.qubits() {
            new_vector_clock[q] = idx;
        }
        new_vector_clock
    }
    pub fn insert_at(&mut self, indices: Vec<(QubitIndex, GateIndex)>, gate: Gate) -> GateIndex {
        //indices should be a complete set of nodes that the gate is connected to
        let new_gate_idx = self.graph.add_gate(GateNode {
            gate: gate.clone(),
            vector_clock: Vec::new(),
        });

        for (qubit_idx, prev_idx) in indices.iter() {
            let succ_idx = self.graph.succ_neighbor_qubit(*prev_idx, *qubit_idx);
            self.graph.remove_edge(*prev_idx, succ_idx, *qubit_idx);
            self.graph.add_edge(*prev_idx, new_gate_idx, *qubit_idx);
            self.graph.add_edge(new_gate_idx, succ_idx, *qubit_idx);
        }

        self.graph.get_gate_mut(new_gate_idx).vector_clock =
            self.get_new_vector_clock(new_gate_idx);
        //Perform BFS and update all vector clocks

        let frontier = self.graph.succ_neighbors(new_gate_idx);
        let mut frontier: HashSet<_> = frontier.iter().map(|(_, idx)| *idx).collect();
        while let Some(gate_idx) = frontier.iter().next().cloned() {
            frontier.remove(&gate_idx);
            let new_vector_clock = self.update_vector_clock(gate_idx);
            let old_vector_clock = self.graph.get_gate(gate_idx).vector_clock.clone();
            if new_vector_clock != old_vector_clock {
                self.graph.get_gate_mut(gate_idx).vector_clock = new_vector_clock;
                let succ_neighbors = self.graph.succ_neighbors(gate_idx);
                for (_, succ_idx) in succ_neighbors {
                    frontier.insert(succ_idx);
                }
            }
        }
        new_gate_idx
    }
    pub fn delete_at(&mut self, idx: GateIndex) {
        let qubits = self.graph.get_gate(idx).gate.qubits();
        let pred_neighbors = self.graph.pred_neighbors(idx);
        let succ_neighbors = self.graph.succ_neighbors(idx);

        let mut pred_succ_pairs: Vec<(QubitIndex, GateIndex, GateIndex)> = Vec::new();
        for q in self.graph.get_gate(idx).gate.qubits() {
            let mut pred_idx = 0;
            let mut succ_idx = 0;
            for (qubit, pred_neighbor) in &pred_neighbors {
                if *qubit == q {
                    pred_idx = *pred_neighbor;
                }
            }
            for (qubit, succ_neighbor) in &succ_neighbors {
                if *qubit == q {
                    succ_idx = *succ_neighbor;
                }
            }
            pred_succ_pairs.push((q, pred_idx, succ_idx));
        }
        self.graph.remove_gate(idx);
        let mut frontier = HashSet::new();
        for (q, pred_idx, succ_idx) in pred_succ_pairs {
            self.graph.add_edge(pred_idx, succ_idx, q);
            frontier.insert(succ_idx);
        }
        // Perform BFS and update all vector clocks

        while let Some(gate_idx) = frontier.iter().next().cloned() {
            frontier.remove(&gate_idx);
            let new_vector_clock = self.update_vector_clock(gate_idx);
            let old_vector_clock = self.graph.get_gate(gate_idx).vector_clock.clone();
            if new_vector_clock != old_vector_clock {
                self.graph.get_gate_mut(gate_idx).vector_clock = new_vector_clock;
                let succ_neighbors = self.graph.succ_neighbors(gate_idx);
                for (_, succ_idx) in succ_neighbors {
                    frontier.insert(succ_idx);
                }
            }
        }
        for q in qubits {
            self.index_priority_map.remove(&(idx, q));
        }
    }
    pub fn make_convex(&self, indices: Vec<GateIndex>) -> Vec<GateIndex> {
        let mut indices = indices;
        loop {
            let new_indices = self.make_convex_add_one(indices.clone());
            if new_indices.len() == indices.len() {
                return new_indices;
            } else {
                indices = new_indices;
            }
        }
    }
    pub fn get_subgraph(&self, indices: &Vec<GateIndex>) -> Vec<Gate> {
        let mut subgraph = DAG::new();
        let mut old_idx_new_idx: HashMap<GateIndex, GateIndex> = HashMap::new();
        for idx in indices {
            let gate = self.graph.get_gate(*idx).gate.clone();
            let new_idx = subgraph.add_gate(GateNode {
                gate,
                vector_clock: vec![],
            });
            old_idx_new_idx.insert(*idx, new_idx);
        }
        for idx in indices {
            let neighbors = self.graph.succ_neighbors(*idx);
            for (qubit, succ_idx) in neighbors {
                if indices.contains(&succ_idx) {
                    subgraph.add_edge(
                        *old_idx_new_idx.get(idx).unwrap(),
                        *old_idx_new_idx.get(&succ_idx).unwrap(),
                        qubit,
                    );
                }
            }
        }
        subgraph.to_gate_vec()
    }
    pub fn get_frontier(&self, indices: Vec<GateIndex>) -> HashMap<QubitIndex, GateIndex> {
        let mut frontier = HashMap::new();
        for index in indices.iter() {
            let pred_neighbors = self.graph.pred_neighbors(*index);
            for (qubit, pred_neighbors) in pred_neighbors {
                if indices.contains(&pred_neighbors) {
                    continue;
                } else {
                    frontier.insert(qubit, pred_neighbors);
                }
            }
        }
        frontier
    }
    pub fn make_convex_add_one(&self, indices: Vec<GateIndex>) -> Vec<GateIndex> {
        // Figure out the incoming and outgoing edges
        let mut outgoing_edges: Vec<Edge> = Vec::new();
        let mut incoming_edges: Vec<Edge> = Vec::new();
        for index in indices.iter() {
            let pred_neighbors = self.graph.pred_neighbors(*index);
            for (qubit, pred_neighbors) in pred_neighbors {
                if indices.contains(&pred_neighbors) {
                    continue;
                } else {
                    incoming_edges.push(Edge {
                        start: pred_neighbors,
                        end: *index,
                        qubit,
                    });
                }
            }
        }
        for index in indices.iter() {
            let succ_neighbors = self.graph.succ_neighbors(*index);
            for (qubit, succ_neighbors) in succ_neighbors {
                if indices.contains(&succ_neighbors) {
                    continue;
                } else {
                    outgoing_edges.push(Edge {
                        start: *index,
                        end: succ_neighbors,
                        qubit,
                    });
                }
            }
        }
        for out_edge in outgoing_edges.iter() {
            for in_edge in incoming_edges.iter() {
                if out_edge.qubit == in_edge.qubit
                    && self.index_priority_map[&(out_edge.start, out_edge.qubit)]
                        < self.index_priority_map[&(in_edge.end, in_edge.qubit)]
                {
                    let mut indices = indices.clone();
                    indices.push(out_edge.end);
                    return indices;
                }
            }
        }
        indices.clone()
    }

    pub fn replace_gates_convex(
        &mut self,
        indices: Vec<GateIndex>,
        new_gates: Vec<Gate>,
    ) -> Vec<GateIndex> {
        let mut frontier = self.get_frontier(indices.clone());
        for idx in indices.iter() {
            self.delete_at(*idx);
        }
        let mut new_gate_indices = vec![];
        for gate in new_gates {
            let qubits = gate.qubits();
            let mut indices = vec![];
            for q in qubits.iter() {
                let pred_idx = *frontier.get(q).unwrap();
                indices.push((*q, pred_idx));
            }
            let new_idx = self.insert_at(indices, gate);
            for q in qubits.iter() {
                frontier.insert(*q, new_idx);
            }
            new_gate_indices.push(new_idx);
        }
        new_gate_indices
    }
    pub fn depth(&self) -> usize {
        let gates = self.graph.to_gate_vec();
        let mut frontier = vec![0; self.num_qubits];
        for gate in gates {
            let qubits = gate.qubits();
            let max_depth = qubits.iter().map(|q| frontier[*q]).max().unwrap();
            qubits.iter().for_each(|q| {
                frontier[*q] = max_depth + 1;
            });
        }
        *frontier.iter().max().unwrap()
    }
    pub fn gate_count(&self) -> usize {
        self.graph.node_count() - 2
    }
    pub fn to_rz(&self) -> Self {
        todo!()
    }
    pub fn cost(&self, cost: &Cost) -> f64 {
        match cost {
            Cost::Depth => self.depth() as f64,
            Cost::Gate => self.gate_count() as f64,
            Cost::Mixed => self.depth() as f64 + 0.1 * self.gate_count() as f64,
        }
    }
    pub fn get_gateset(&self) -> Gateset {
        let mut gateset = Gateset::Nam;
        for node in self.graph.node_weights() {
            match node.gate {
                Gate::CCX { .. } => {
                    gateset = Gateset::CliffordT;
                }
                Gate::CCZ { .. } => {
                    gateset = Gateset::CliffordT;
                }
                _ => {}
            }
        }
        gateset
    }
    pub fn to_seq(&self) -> CircuitSeq {
        CircuitSeq::new(self.graph.to_gate_vec(), self.num_qubits)
    }
}

#[cfg(test)]
mod tests {
    use crate::CircuitSeq;

    use super::*;
    #[test]
    fn test_dag() {
        let circ_seq =
            CircuitSeq::new_from_source("qreg q[2];\nH q[0];\nH q[0];\nCX q[0], q[1];\n");
        let dag = CircuitDag::new(circ_seq.gates, circ_seq.num_qubits);
        assert_eq!(dag.num_qubits, 2);
        assert_eq!(dag.graph.node_count(), 5);
        println!("{:?}", dag.graph);
        assert_eq!(dag.graph.edge_count(), 6);
        assert_eq!(dag.depth(), 3);
        println!("{}", dag.to_seq().dump());
    }
    #[test]
    fn test_dag_2() {
        let gates = vec![Gate::H(0), Gate::H(0), Gate::CX { q1: 0, q2: 1 }];
        let dag = CircuitDag::new(gates, 2);
        assert_eq!(dag.num_qubits, 2);
        assert_eq!(dag.graph.node_count(), 5);
        println!("{:?}", dag.graph);
        assert_eq!(dag.graph.edge_count(), 6);
        assert_eq!(dag.depth(), 3);
        println!("{}", dag.to_seq().dump());
    }
    #[test]
    fn test_new() {
        let gates = vec![Gate::H(0), Gate::X(1), Gate::CX { q1: 0, q2: 1 }];
        let num_qubits = 2;
        let dag = CircuitDag::new(gates.clone(), num_qubits);

        println!("{:?}", dag.graph);
    }

    #[test]
    fn test_insert_at() {
        let mut dag = CircuitDag::new(vec![Gate::H(0), Gate::X(1)], 2);

        dag.insert_at(vec![(0, 0), (1, 0)], Gate::CX { q1: 0, q2: 1 });
        assert_eq!(dag.graph.node_count(), 5);
        println!("{:?}", dag.graph.to_gate_vec());
        assert!(dag.graph.contains_edge(4, 2));
        assert!(dag.graph.contains_edge(4, 3));
        assert!(!dag.graph.contains_edge(0, 2));
    }

    #[test]
    fn test_delete_at() {
        let mut dag = CircuitDag::new(vec![Gate::H(0), Gate::X(1), Gate::CX { q1: 0, q2: 1 }], 2);
        let cx_node = 4;

        dag.delete_at(cx_node);
        assert_eq!(dag.graph.node_count(), 4);
        assert!(dag.graph.contains_edge(2, 1));
        assert!(dag.graph.contains_edge(3, 1));
    }

    #[test]
    fn test_depth_gate_count_cost() {
        let dag = CircuitDag::new(vec![Gate::H(0), Gate::X(1), Gate::CX { q1: 0, q2: 1 }], 2);

        assert_eq!(dag.depth(), 2);
        assert_eq!(dag.gate_count(), 3);
        assert_eq!(dag.cost(&Cost::Depth), 2.0);
        assert_eq!(dag.cost(&Cost::Gate), 3.0);
        assert_eq!(dag.cost(&Cost::Mixed), 2.3);
    }

    #[test]
    fn test_dag_dump() {
        let dag1 = CircuitDag::new(vec![Gate::H(0), Gate::X(1), Gate::CX { q1: 0, q2: 1 }], 2);
        let qasm = dag1.to_seq().dump();
        let circ_seq = CircuitSeq::new_from_source(&qasm);
        let dag2 = CircuitDag::new(circ_seq.gates, circ_seq.num_qubits);
        assert_eq!(dag1.num_qubits, dag2.num_qubits);
        assert_eq!(dag1.graph.node_count(), dag2.graph.node_count());
        assert_eq!(dag1.depth(), dag2.depth());
    }
    #[test]
    fn test_convexify() {
        let mut dag = CircuitDag::new(
            vec![
                Gate::CX { q1: 0, q2: 1 },
                Gate::H(0),
                Gate::H(1),
                Gate::CX { q1: 0, q2: 1 },
            ],
            2,
        );
        let indices = vec![2, 3, 5];
        let new_indices = dag.make_convex(indices);
        let gates = dag.get_subgraph(&new_indices);
        println!("{:?}", new_indices);
        dag.replace_gates_convex(new_indices, gates);
        println!("{:?}", dag.to_seq().dump());
    }
    #[test]
    fn test_integration() {
        let mut dag = CircuitDag::new(
            vec![
                Gate::CX { q1: 0, q2: 1 },
                Gate::H(0),
                Gate::X(1),
                Gate::CZ { q1: 0, q2: 1 },
            ],
            2,
        );
        let mut counter = 0;
        while let Some(node) = dag.graph.next_unoptimized_gate() {
            let indices = dag.graph.get_neighbors(node, 1);
            println!("+++++");
            println!("{:?}", dag.graph);
            println!(
                "{:?}",
                indices
                    .iter()
                    .map(|i| dag.graph.get_gate(*i).gate.clone())
                    .collect::<Vec<_>>()
            );
            println!("{:?}", indices);
            println!("{:?}", dag.index_priority_map.keys());
            let new_indices = dag.make_convex(indices);
            let gates = dag.get_subgraph(&new_indices);
            println!("{:?}", new_indices);
            dag.replace_gates_convex(new_indices, gates);
            counter += 1;
            if counter > 100 {
                break;
            }
            // dag.graph.set_optimized(node);
        }
    }
}
