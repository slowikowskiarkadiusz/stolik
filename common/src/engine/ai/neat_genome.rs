extern crate alloc;
use core::sync::atomic::AtomicU32;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    vec::Vec,
};
use libm::exp;
use rand::{Rng, rngs::SmallRng};

use crate::engine::ai::innovation_tracker::{get_innovation_id, get_next_node_id};
use crate::engine::input::key::KEYS_LENGTH;
use serde::{Deserialize, Serialize};

pub type Id = u32;
pub type AtomicId = AtomicU32;

pub struct DataForAi {
    pub inputs: [Vec<f64>; 2],
    pub points: [f64; 2],
    pub is_gameover: bool,
    pub outputs_to_keys: fn(&[f64]) -> [bool; KEYS_LENGTH as usize],
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Connection {
    from: Id,
    to: Id,
    weight: f64,
    is_enabled: bool,
    innovation: Id,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
enum NodeType {
    Input,
    Output,
    Hidden,
    Bias,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NeatGenome {
    nodes: BTreeMap<Id, NodeType>,
    connections: Vec<Connection>,
    pub fitness: f64,
    species_id: Id,

    inputs_count: u32,
    outputs_count: u32,
}

impl NeatGenome {
    pub fn reproduce(mut genomes: Vec<NeatGenome>, population_size: u8, rng: &mut SmallRng) -> Vec<NeatGenome> {
        genomes.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(core::cmp::Ordering::Equal));

        let mut new_population = Vec::<NeatGenome>::new();

        // elity — najlepsze 2 przechodzą bez zmian
        new_population.push(genomes[0].clone());
        new_population.push(genomes[1].clone());

        let tournament_size = (population_size / 3).max(2);
        while new_population.len() < population_size as usize {
            let a = tournament_select(&genomes, tournament_size, rng);
            let b = tournament_select(&genomes, tournament_size, rng);
            new_population.push(bara_bara(&genomes[a], &genomes[b], rng));
        }

        new_population
    }

    pub fn new(inputs_count: u32, outputs_count: u32) -> Self {
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

        let mut connections: Vec<Connection> = Vec::new();

        for input_node in input_nodes.iter() {
            for output_node in output_nodes.iter() {
                connections.push(Connection {
                    from: input_node.0.clone(),
                    to: output_node.0.clone(),
                    weight: 0.0,
                    is_enabled: true,
                    innovation: get_innovation_id(input_node.0.clone(), output_node.0.clone()),
                });
            }
        }

        input_nodes.append(&mut output_nodes);

        Self {
            nodes: input_nodes,
            connections,
            fitness: 0.0,
            species_id: 0,
            inputs_count,
            outputs_count,
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

        for i in 0..inputs.len().min(input_nodes.len()) {
            node_values.insert(input_nodes[i].0, inputs[i]);
        }

        for node_id in self.topological_sort() {
            if self.nodes[&node_id] == NodeType::Input {
                continue;
            }

            let mut sum: f64 = 0.0;

            for connection in self.connections.iter().filter(|f| f.to == node_id && f.is_enabled) {
                sum += node_values.get(&connection.from).copied().unwrap_or(0.0) * connection.weight;
            }

            node_values.insert(node_id, sigmoid(sum));
        }

        output_nodes.iter().map(|f| node_values[&f.0]).collect()
    }

    fn topological_sort(&self) -> Vec<Id> {
        let mut visited = BTreeMap::<Id, bool>::new();
        let mut result = Vec::<Id>::new();

        fn visit(node_id: Id, visited: &mut BTreeMap<Id, bool>, connections: &[Connection], result: &mut Vec<Id>) {
            if visited.get(&node_id).is_some() {
                return;
            }

            visited.insert(node_id, true);

            for next in connections.iter().filter(|f| f.to == node_id && f.is_enabled) {
                visit(next.from, visited, connections, result);
            }

            result.push(node_id);
        }

        for node in self.nodes.iter().filter(|f| f.1 == &NodeType::Output) {
            visit(node.0.clone(), &mut visited, &self.connections, &mut result);
        }

        result
    }

    pub fn mutate(&mut self, rng: &mut SmallRng) {
        if rng.gen_range(0..=100) < 3 {
            self.mutate_add_node(rng);
        }
        if rng.gen_range(0..=100) < 5 {
            self.mutate_add_connection(rng);
        }
        if rng.gen_range(0..=100) < 80 {
            self.mutate_weights(rng);
        }
        if rng.gen_range(0..=100) < 1 {
            self.mutate_toggle_enabled(rng);
        }
    }

    fn mutate_add_node(&mut self, rng: &mut SmallRng) {
        let enabled_nodes: Vec<Connection> = { self.connections.iter().filter(|f| f.is_enabled).map(|f| f.clone()).collect() };
        let enabled_nodes_len = enabled_nodes.len();

        if enabled_nodes_len == 0 {
            return;
        }

        let mut random_connection = enabled_nodes[rng.gen_range(0..enabled_nodes_len)];
        random_connection.is_enabled = false;

        let new_node_id = get_next_node_id();
        self.nodes.insert(new_node_id, NodeType::Hidden);

        let innovation_from = get_innovation_id(random_connection.from, new_node_id);
        let innovation_to = get_innovation_id(random_connection.to, new_node_id);

        self.connections.insert(
            0,
            Connection {
                from: random_connection.from,
                to: new_node_id,
                weight: 1.0,
                is_enabled: true,
                innovation: innovation_from,
            },
        );

        self.connections.insert(
            0,
            Connection {
                from: new_node_id,
                to: random_connection.to,
                weight: 1.0,
                is_enabled: true,
                innovation: innovation_to,
            },
        );
    }

    fn mutate_add_connection(&mut self, rng: &mut SmallRng) {
        fn would_create_cycle(connections: &[Connection], from: &Id, to: &Id) -> bool {
            let mut visited: BTreeSet<Id> = BTreeSet::new();
            let mut stack: Vec<Id> = Vec::new();
            stack.push(to.clone());

            while stack.len() > 0 {
                let current = stack.pop().unwrap();
                if &current == from {
                    return true;
                }
                if !visited.insert(current) {
                    continue;
                }

                for connection in connections.iter().filter(|f| f.is_enabled && f.from == current) {
                    stack.push(connection.to);
                }
            }

            false
        }

        let non_outputs: Vec<&Id> = self.nodes.iter().filter(|f| f.1 != &NodeType::Output).map(|f| f.0).collect();
        let node_from = non_outputs[rng.gen_range(0..non_outputs.len())];

        let non_inputs: Vec<&Id> = self.nodes.iter().filter(|f| f.1 != &NodeType::Input).map(|f| f.0).collect();
        let node_to = non_inputs[rng.gen_range(0..non_inputs.len())];

        if node_from == node_to {
            return;
        }

        if would_create_cycle(&self.connections, node_from, node_to)
            || self.connections.iter().any(|f| &f.from == node_from && &f.to == node_to)
        {
            return;
        }

        // TODO node id powinno byc genome-scoped
        self.connections.push(Connection {
            from: node_from.clone(),
            to: node_to.clone(),
            weight: rng.gen_range(-1.0..=1.0),
            is_enabled: true,
            innovation: get_innovation_id(node_from.clone(), node_to.clone()),
        });
    }

    fn mutate_weights(&mut self, rng: &mut SmallRng) {
        for connection in &mut self.connections {
            if rng.gen_range(0..=10) < 90 {
                connection.weight += (rng.gen_range(0.0..1.0) * 2.0 - 1.0) * 0.5;
            } else {
                connection.weight = rng.gen_range(-1.0..1.0);
            }
        }
    }

    fn mutate_toggle_enabled(&mut self, rng: &mut SmallRng) {
        if self.connections.len() == 0 {
            return;
        }

        let mut random_connection = self.connections[rng.gen_range(0..self.connections.len())];
        random_connection.is_enabled = !random_connection.is_enabled;
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + exp(-4.9 * x))
}

impl NeatGenome {
    pub fn from_json(json: &str) -> Option<Self> {
        if json.trim().is_empty() {
            return None;
        }
        serde_json::from_str(json).ok()
    }

    pub fn to_json(&self) -> alloc::string::String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

fn bara_bara(parent1: &NeatGenome, parent2: &NeatGenome, rng: &mut SmallRng) -> NeatGenome {
    let mut nodes = BTreeMap::<Id, NodeType>::new();

    let better = if parent1.fitness >= parent2.fitness { parent1 } else { parent2 };
    let worse = if parent1.fitness >= parent2.fitness { parent2 } else { parent1 };

    for better_node in &better.nodes {
        nodes.insert(better_node.0.clone(), better_node.1.clone());
    }

    let mut new_connections: Vec<Connection> = Vec::new();

    for connection in &better.connections {
        let find_matching = worse.connections.iter().find(|f| f.innovation == connection.innovation);

        let new_connection = if let Some(matching) = find_matching
            && rng.gen_range(0..2) < 1
        {
            Connection {
                from: connection.from,
                to: connection.to,
                weight: matching.weight,
                is_enabled: matching.is_enabled,
                innovation: connection.innovation,
            }
        } else {
            connection.clone()
        };

        new_connections.push(new_connection);
    }

    let mut child = NeatGenome {
        nodes: nodes,
        connections: new_connections,
        fitness: 0.0,
        species_id: 0,
        inputs_count: parent1.inputs_count,
        outputs_count: parent1.outputs_count,
    };

    child.mutate(rng);

    child
}

fn tournament_select(genomes: &[NeatGenome], tournament_size: u8, rng: &mut SmallRng) -> usize {
    let mut best: Option<usize> = None;

    for i in 0..tournament_size {
        let candidate_index = rng.gen_range(0..genomes.len());
        if best.is_none() || genomes[candidate_index].fitness > genomes[best.unwrap().clone()].fitness {
            best = Some(candidate_index);
        }
    }

    best.unwrap().clone()
}
