use std::fmt;

use clap::{ArgEnum, Parser};
use csv::{self, StringRecord};
use fast_paths::{FastGraph, InputGraph};
use pathfinder::PathfinderGraph;
use serde::{self, de, Deserialize, Deserializer};

use crate::pathfinder::into_pathfinder_graph;

mod pathfinder;

const MAX_WEIGHT_VALUE: f32 = 4294967296_f32;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    input: String,

    #[clap(short, long)]
    output: String,

    #[clap(short, long, arg_enum, default_value = "fast-path")]
    algorithm: Algorithm,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
enum Algorithm {
    Dijkstra,
    FastPath,
}

#[derive(Clone, Debug, Deserialize)]
struct WeightedNodes {
    #[serde(deserialize_with = "deserialize_int_or_float")]
    src: u64,
    #[serde(deserialize_with = "deserialize_int_or_float")]
    dst: u64,
    weight: f32,
}

#[derive(Clone, Debug)]
struct ShortestPathLength {
    src: usize,
    dst: usize,
    length: usize,
}

fn into_input_graph(weighted_nodes: Vec<WeightedNodes>) -> InputGraph {
    let mut input_graph = InputGraph::new();
    for node in weighted_nodes.iter() {
        input_graph.add_edge_bidir(node.src as usize, node.dst as usize, (node.weight) as usize);
    }
    input_graph.freeze();
    input_graph
}

struct DeserializeFloatOrIntegerVisitor;

impl<'de> de::Visitor<'de> for DeserializeFloatOrIntegerVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or a float")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v as u64)
    }
}

fn deserialize_int_or_float<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(DeserializeFloatOrIntegerVisitor)
}

fn read_weighted_nodes(path: &str) -> Vec<WeightedNodes> {
    let mut reader = csv::Reader::from_path(path).unwrap();
    reader.set_headers(StringRecord::from(vec!["src", "dst", "weight"]));
    // for result in reader.records() {
    //     println!("{:?}", result.unwrap());
    // }
    reader.deserialize().map(|x| x.unwrap()).collect()
}

fn all_pairs_path_length(graph: FastGraph) -> Vec<ShortestPathLength> {
    let mut res = Vec::with_capacity(graph.get_num_nodes() * graph.get_num_nodes());
    let mut path_calculator = fast_paths::create_calculator(&graph);
    for src in 0..graph.get_num_nodes() {
        for dst in 0..graph.get_num_nodes() {
            match path_calculator.calc_path(&graph, src, dst) {
                Some(path) => res.push(ShortestPathLength {
                    src,
                    dst,
                    length: path.get_weight(),
                }),
                None => (),
            }
        }
    }
    res
}

fn all_pairs_path_length_pathfinder(graph: PathfinderGraph) -> Vec<ShortestPathLength> {
    let mut res = Vec::with_capacity(graph.num_nodes * graph.num_nodes);

    for src in 0..graph.num_nodes {
        for (dst, (_, weight)) in graph.all_paths_for_node(src) {
            res.push(ShortestPathLength {
                src,
                dst,
                length: weight,
            })
        }
    }
    res
}

fn write_shortest_paths(output: &str, paths: Vec<ShortestPathLength>) {
    let mut writer = csv::Writer::from_path(output).unwrap();

    for path in paths {
        writer
            .write_record(&[
                path.src.to_string(),
                path.dst.to_string(),
                (path.length as f32).to_string(),
            ])
            .unwrap();
    }
}

fn main() {
    let args = Args::parse();
    let paths = match args.algorithm {
        Algorithm::FastPath => {
            let nodes = read_weighted_nodes(&args.input);
            let input_graph = into_input_graph(nodes);
            let fast_graph = fast_paths::prepare(&input_graph);
            all_pairs_path_length(fast_graph)
        }
        Algorithm::Dijkstra => {
            let nodes = read_weighted_nodes(&args.input);
            let graph = into_pathfinder_graph(nodes);
            all_pairs_path_length_pathfinder(graph)
        }
    };
    write_shortest_paths(&args.output, paths)
}
