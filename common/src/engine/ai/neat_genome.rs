extern crate alloc;
use alloc::{collections::BTreeMap, vec::Vec};
use libm::exp;

type Id = u32;

struct Connection {
    from: Id,
    to: Id,
    weight: f64,
    is_enabled: bool,
    innovation: Id,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum NodeType {
    Input,
    Output,
    Hidden,
    Bias,
}

struct NeatGenome {
    id: Id,
    nodes: BTreeMap<Id, NodeType>,
    connections: BTreeMap<Id, Connection>,
    fitness: f64,
    adjusted_fitness: f64,
    species_id: Id,
}

impl NeatGenome {
    pub fn new(id: Id, inputs_count: u32, outputs_count: u32) -> Self {
        let mut input_nodes: BTreeMap<Id, NodeType> = BTreeMap::new();
        let mut output_nodes: BTreeMap<Id, NodeType> = BTreeMap::new();

        let mut node_id: Id = 0;
        for _ in 0..inputs_count {
            input_nodes.insert(node_id, NodeType::Input);
            node_id += 1;
        }

        for _ in 0..outputs_count {
            output_nodes.insert(node_id, NodeType::Output);
            node_id += 1;
        }

        let mut connections: BTreeMap<Id, Connection> = BTreeMap::new();

        let mut connection_id: Id = 0;
        for input_node in input_nodes.iter() {
            for output_node in output_nodes.iter() {
                connections.insert(
                    connection_id,
                    Connection {
                        from: input_node.0.clone(),
                        to: output_node.0.clone(),
                        weight: 0.0,
                        is_enabled: true,
                        innovation: get_innovation_id(input_node.0.clone(), output_node.0.clone()),
                    },
                );
                connection_id += 1;
            }
        }

        input_nodes.append(&mut output_nodes);

        Self {
            id,
            nodes: input_nodes,
            connections,
            fitness: 0.0,
            adjusted_fitness: 0.0,
            species_id: 0,
        }
    }

    pub fn activate(&mut self, inputs: Vec<f64>) -> Vec<f64> {
        let mut node_values = BTreeMap::<Id, f64>::new();

        // to moze byc jedna petla
        let input_nodes: Vec<(Id, NodeType)> = self
            .nodes
            .iter()
            .filter(|f| f.1 == &NodeType::Input)
            .map(|f| (f.0.clone(), f.1.clone()))
            .collect();
        let output_nodes: Vec<(Id, NodeType)> = self
            .nodes
            .iter()
            .filter(|f| f.1 == &NodeType::Output)
            .map(|f| (f.0.clone(), f.1.clone()))
            .collect();

        for i in 0..inputs.len() {
            node_values.insert(input_nodes[i].0, inputs[i]);
        }

        for node_id in self.topological_sort() {
            if self.nodes[&node_id] == NodeType::Input {
                continue;
            }

            let mut sum: f64 = 0.0;

            for connection in self.connections.iter().map(|f| f.1).filter(|f| f.to == node_id && f.is_enabled) {
                sum += node_values[&connection.from] * connection.weight;
            }

            node_values.insert(node_id, sigmoid(sum));
        }

        output_nodes.iter().map(|f| node_values[&f.0]).collect()
    }

    fn topological_sort(&self) -> Vec<Id> {
        let mut visited = BTreeMap::<Id, bool>::new();
        let mut result = Vec::<Id>::new();

        fn visit(node_id: Id, visited: &mut BTreeMap<Id, bool>, connections: &BTreeMap<Id, Connection>, result: &mut Vec<Id>) {
            if visited.get(&node_id).is_some() {
                return;
            }

            visited.insert(node_id, true);

            for next in connections.iter().filter(|f| f.1.to == node_id && f.1.is_enabled) {
                visit(next.1.from, visited, connections, result);
            }

            result.push(node_id);
        }

        for node in self.nodes.iter().filter(|f| f.1 == &NodeType::Output) {
            visit(node.0.clone(), &mut visited, &self.connections, &mut result);
        }

        result
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + exp(-4.9 * x))
}

fn get_innovation_id(input_id: Id, output_id: Id) -> Id {
    0
}
