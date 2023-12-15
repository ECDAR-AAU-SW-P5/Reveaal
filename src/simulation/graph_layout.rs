use crate::data_reader::serialization::DummyComponent;
use force_graph::{DefaultNodeIdx, ForceGraph, NodeData};
use log::info;
use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Configuration for component generation //TODO: use GRPC to set.
#[derive(Debug, Deserialize)]
struct Config {
    /// Padding of sides of component
    padding: f32,
    /// Square space needed for each location
    location_space: f32,
    /// Maximal aspect ratio of the component
    max_ratio: f32,
    location_mass: f32,
    edge_mass: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            padding: 100.0,
            location_space: 200.0,
            max_ratio: 1.5,
            location_mass: 1.0,
            edge_mass: 1.0,
        }
    }
}

fn get_config() -> Config {
    match read_config("config.json") {
        Ok(config) => config,
        Err(_) => {
            info!("Could not find graph layout config, using defaults");
            Config::default()
        }
    }
}

fn read_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;

    Ok(u)
}

// Data object for force graph
struct Data {
    pub location: bool,
    pub location_number: usize,
    pub edge_number: usize,
    pub nail_number: usize,
}

pub fn layout_dummy_component(comp: &mut DummyComponent) {
    let config = get_config();

    // Compute the grid size
    let locs = comp.locations.len();
    let loc_sqrt = (locs as f32).sqrt();
    let grid_size = f32::max(loc_sqrt * config.location_space, 200.0);

    // Construct the force graph with nodes in random locations
    let mut graph = <ForceGraph<Data>>::new(Default::default());

    let mut rng = rand::thread_rng();
    let range_max = 1000.0;
    let range_min = -range_max;

    // Keep track of location node ids
    let mut node_map: HashMap<String, DefaultNodeIdx> = HashMap::new();

    // We make the first node an anchor
    let mut first = true;

    for (i, location) in comp.locations.iter().enumerate() {
        let node = graph.add_node(NodeData {
            x: rng.gen_range(range_min..range_max),
            y: rng.gen_range(range_min..range_max),
            is_anchor: first,
            user_data: Data {
                location: true,
                location_number: i,
                edge_number: 0,
                nail_number: 0,
            },
            mass: config.location_mass,
        });

        node_map.insert(location.id.clone(), node);

        first = false;
    }

    // Iterate over nails in edges and treat them as nodes
    for (i, edge) in comp.edges.iter().enumerate() {
        let mut first = None;
        let mut last = None;

        for (j, _) in edge.nails.iter().enumerate() {
            let node = graph.add_node(NodeData {
                x: rng.gen_range(range_min..range_max),
                y: rng.gen_range(range_min..range_max),
                is_anchor: false,
                user_data: Data {
                    location: false,
                    location_number: 0,
                    edge_number: i,
                    nail_number: j,
                },
                mass: config.edge_mass,
            });

            if first.is_none() {
                first = Some(node);
            }

            if let Some(other) = last {
                graph.add_edge(node, other, Default::default());
            }

            last = Some(node);
        }

        if first.and(last).is_some() {
            graph.add_edge(
                first.unwrap(),
                *node_map.get(&edge.source_location).unwrap(),
                Default::default(),
            );
            graph.add_edge(
                last.unwrap(),
                *node_map.get(&edge.target_location).unwrap(),
                Default::default(),
            );
        }

        // Add additional edge between locations
        graph.add_edge(
            *node_map.get(&edge.source_location).unwrap(),
            *node_map.get(&edge.target_location).unwrap(),
            Default::default(),
        );
    }

    // Run the force based simulation
    for _ in 0..1000 {
        graph.update(0.01);
    }

    // Compute the bounds on the node coordinates so we can normalize
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;

    graph.visit_nodes(|node| {
        min_x = f32::min(node.x(), min_x);
        min_y = f32::min(node.y(), min_y);
        max_x = f32::max(node.x(), max_x);
        max_y = f32::max(node.y(), max_y);
    });

    // Normalize bounds to grid size
    let normalize = {
        |num: f32, min: f32, max: f32, ratio: f32| {
            ((num - min) / (max - min)) * (grid_size * ratio) + config.padding / 2.0
        }
    };

    // Ensure the aspect ratio
    fn clamp(num: f32, min: f32, max: f32) -> f32 {
        f32::max(f32::min(num, max), min)
    }

    let ratio_x = clamp(
        (max_x - min_x) / (max_y - min_y),
        1.0 / config.max_ratio,
        config.max_ratio,
    );

    let ratio_y = 1.0 / ratio_x;

    let normalize_x = { |num: f32| normalize(num, min_x, max_x, ratio_x) };
    let normalize_y = { |num: f32| normalize(num, min_y, max_y, ratio_y) };

    // Set the location and nail coordinates
    graph.visit_nodes(|node| {
        let data = &node.data.user_data;
        if data.location {
            comp.locations[data.location_number].x = normalize_x(node.x());
            comp.locations[data.location_number].y = normalize_y(node.y());
        } else {
            comp.edges[data.edge_number].nails[data.nail_number].x = normalize_x(node.x());
            comp.edges[data.edge_number].nails[data.nail_number].y = normalize_y(node.y());
        }
    });

    // Set the component shape
    comp.width = grid_size * ratio_x + config.padding;
    comp.height = grid_size * ratio_y + config.padding;

    // Translate so it is centered
    comp.x = -comp.width / 2.0;
    comp.y = -comp.height / 2.0;

    // Choose a random color for the component
    let color = rng.gen_range(0..10);
    comp.color = color.to_string();

    // Apply the color to all locations as well
    for loc in &mut comp.locations {
        loc.color = color;
    }
}
