use pathfinding::prelude::dijkstra_all;
use std::collections::HashMap;

use crate::WeightedNodes;

pub(crate) struct PathfinderGraph {
    successors: HashMap<usize, Vec<(usize, usize)>>,
    pub(crate) num_nodes: usize,
}

impl PathfinderGraph {
    pub(crate) fn all_paths_for_node(&self, src: usize) -> HashMap<usize, (usize, usize)> {
        dijkstra_all(&src, |node| successors(node, self))
    }
}

fn successors(src: &usize, graph: &PathfinderGraph) -> Vec<(usize, usize)> {
    if graph.successors.contains_key(src) {
        graph.successors.get(src).unwrap().clone()
    } else {
        Vec::new()
    }
}

fn insert(map: &mut HashMap<usize, Vec<(usize, usize)>>, src: u64, dst: u64, weight: f32) {
    let key = src as usize;
    if !map.contains_key(&key) {
        map.insert(src as usize, Vec::new());
    }
    map.get_mut(&key)
        .unwrap()
        .push((dst as usize, weight as usize))
}

pub(crate) fn into_pathfinder_graph(weighted_nodes: Vec<WeightedNodes>) -> PathfinderGraph {
    let mut successors = HashMap::new();

    for node in weighted_nodes {
        insert(&mut successors, node.src, node.dst, node.weight);
        insert(&mut successors, node.dst, node.src, node.weight);
    }
    let num_nodes = successors.keys().count();
    PathfinderGraph {
        successors,
        num_nodes,
    }
}
