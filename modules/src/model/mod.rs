use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataBase {
    pub app_graph: AppGraph,
    pub global_constraint: GlobalConstraint,
    pub alimp_lib: AlimpLibrary,
    pub hyper_parameter: HyperParameter,
    pub technology_constraint: TechConstraint,
    pub routing_graph: RoutingGraph,
    pub floor_plan: FloorPlan,
    pub cost_metric: CostMetric,
    pub alimp_binding_options: Vec<AlimpBindingOption>,
    pub synthesized_information: SynthesizedInformation,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppGraph{
    pub nodes: Vec<AppNode>,
    pub edges: Vec<AppEdge>,
    pub global_mem_image: String,
    pub global_mem_reference: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppNode {
    pub id: String,
    pub func: String,
    #[serde(default)]
    pub input_ports: Vec<AppNodePort>,
    #[serde(default)]
    pub output_ports: Vec<AppNodePort>,
    #[serde(default)]
    pub repetition: i32,
    #[serde(default)]
    pub execution_time: i32,
    #[serde(default)]
    pub executable: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppNodePort {
    pub id: String,
    pub rate: i32,
    #[serde(default)]
    pub token_type: String,
    #[serde(default)]
    pub token_size: i32,
    #[serde(default)]
    pub addr_time_patterns: Vec<PairIntInt>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppEdge {
    pub id: String,
    pub source_node: String,
    pub target_node: String,
    pub source_port: String,
    pub target_port: String,
    #[serde(default)]
    pub token_type: String,
    #[serde(default)]
    pub token_size: i32,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalConstraint {
    pub max_width: i32,
    pub max_height: i32,
    pub max_latency: i32,
    pub max_period: i32,
    pub max_energy: i32,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlimpLibrary {
    pub entries: Vec<AlimpEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlimpEntry {
    pub func: String,
    pub instances: Vec<AlimpInstance>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlimpInstance {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub width: i32,
    #[serde(default)]
    pub height: i32,
    #[serde(default)]
    pub input_port_positions: Vec<i32>,
    #[serde(default)]
    pub output_port_positions: Vec<i32>,
    #[serde(default)]
    pub frequency: i32,
    #[serde(default)]
    pub latency: i32,
    #[serde(default)]
    pub power: i32,
    #[serde(default)]
    pub energy: i32,
    #[serde(default)]
    pub input_addr_time_patterns: Vec<PairIntInt>,
    #[serde(default)]
    pub output_addr_time_patterns: Vec<PairIntInt>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HyperParameter {
    pub bind_w_area: i32,
    pub bind_w_energy: i32,
    pub bind_w_latency: i32,
    pub bind_relaxation_factor: f64,
    pub place_relaxation_factor: f64,
    pub place_reserved_routing_size: i32,
}
 

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TechConstraint {
    pub required_period: i32,
    pub required_slew: i32,
    pub initial_slew: i32,
    pub buffer_slew_declined_factor: i32,
    pub buffer_delay_improved_factor: i32,
    pub register_slew_constant: i32,
    pub number_slew_rates: i32,
    pub number_wire_blocks: i32,
    pub slew_rates: Vec<i32>,
    pub timing_table: Vec<TimingRow>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimingRow {
    pub rows: Vec<i32>,
}
 

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingGraph {
    pub app_edge_id: String,
    pub path: Vec<Coordinate>,
    pub delay: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FloorPlan {
    pub app_node_ids: String,
    pub app_edge_ids: String,
    pub max_width: i32,
    pub max_height: i32,
    pub pos: Vec<RectanglePosition>,
    pub shape: Vec<RectangleShape>,
    pub source_node: Vec<i32>,
    pub target_node: Vec<i32>,
    pub source_port: Vec<i32>,
    pub target_port: Vec<i32>,
    pub conn: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RectanglePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RectangleShape {
    pub width: i32,
    pub height: i32,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostMetric {
    pub width: i32,
    pub height: i32,
    pub area: i32,
    pub energy: i32,
    pub latency: i32,
    pub period: i32,
}
 

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlimpBindingOption {
    pub alimp_bindings: Vec<AlimpBinding>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlimpBinding {
    pub app_node_id: String,
    pub alimp_instance: AlimpInstance,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynthesizedInformation {
    pub alimp_bindings: Vec<AlimpBinding>,
    pub placements: Vec<Placement>,
    pub max_width: i32,
    pub max_height: i32,
    pub routing_paths: Vec<RoutingPath>,
    pub max_latency: i32,
    pub node_fire_times: HashMap<String, i32>,
    pub transporter_fire_times: HashMap<String, i32>,
    pub channel_width: HashMap<String, i32>,
    pub input_buffer_size: HashMap<String, i32>,
    pub output_buffer_size: HashMap<String, i32>,
    pub chunk_address_assignments: Vec<ChunkAddressAssignment>,
    pub wire_assignment: HashMap<String, String>,
    pub transport_tables: HashMap<String, TransportTable>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub app_node_id: String,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingPath {
    pub app_edge_id: String,
    pub path: Vec<Coordinate>,
    pub delay: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkAddressAssignment {
    pub app_node_id: String,
    pub port_id: String,
    pub address_assignment: HashMap<i32, i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportTable {
    pub app_edge_id: String,
    pub entries: Vec<TransportTableEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportTableEntry {
    pub time: i32,
    pub source_address: i32,
    pub target_address: i32,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PairIntInt {
    #[serde(default)]
    pub key: i32,
    #[serde(default)]
    pub value: i32,
}


impl DataBase {
    pub fn new() -> Self {
        Self {
            app_graph: AppGraph {
                nodes: vec![],
                edges: vec![],
                global_mem_image: String::new(),
                global_mem_reference: String::new(),
            },
            global_constraint: GlobalConstraint {
                max_width: 0,
                max_height: 0,
                max_latency: 0,
                max_period: 0,
                max_energy: 0,
            },
            alimp_lib: AlimpLibrary { entries: vec![] },
            hyper_parameter: HyperParameter {
                bind_w_area: 0,
                bind_w_energy: 0,
                bind_w_latency: 0,
                bind_relaxation_factor: 0.0,
                place_relaxation_factor: 0.0,
                place_reserved_routing_size: 0,
            },
            technology_constraint: TechConstraint {
                required_period: 0,
                required_slew: 0,
                initial_slew: 0,
                buffer_slew_declined_factor: 0,
                buffer_delay_improved_factor: 0,
                register_slew_constant: 0,
                number_slew_rates: 0,
                number_wire_blocks: 0,
                slew_rates: vec![],
                timing_table: vec![],
            },
            routing_graph: RoutingGraph {
                app_edge_id: String::new(),
                path: vec![],
                delay: 0,
            },
            floor_plan: FloorPlan {
                app_node_ids: String::new(),
                app_edge_ids: String::new(),
                max_width: 0,
                max_height: 0,
                pos: vec![],
                shape: vec![],
                source_node: vec![],
                target_node: vec![],
                source_port: vec![],
                target_port: vec![],
                conn: vec![],
            },
            cost_metric: CostMetric {
                width: 0,
                height: 0,
                area: 0,
                energy: 0,
                latency: 0,
                period: 0,
            },
            alimp_binding_options: vec![] ,
            synthesized_information: SynthesizedInformation {
                alimp_bindings: vec![],
                placements: vec![],
                max_width: 0,
                max_height: 0,
                routing_paths: vec![],
                max_latency: 0,
                node_fire_times: std::collections::HashMap::new(),
                transporter_fire_times: std::collections::HashMap::new(),
                channel_width: std::collections::HashMap::new(),
                input_buffer_size: std::collections::HashMap::new(),
                output_buffer_size: std::collections::HashMap::new(),
                chunk_address_assignments: vec![],
                wire_assignment: std::collections::HashMap::new(),
                transport_tables: std::collections::HashMap::new(),
            },
        }
    }
}
