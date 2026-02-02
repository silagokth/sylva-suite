use std::collections::{HashMap, BTreeMap};
use serde::{Deserialize, Serialize};
use std::fmt;


#[derive(Debug, Serialize, Deserialize, Clone)]
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


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppGraph{
    pub nodes: Vec<AppNode>,
    pub edges: Vec<AppEdge>,
    pub global_mem_image: String,
    pub global_mem_reference: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppNodePort {
    pub id: String,
    pub rate: i32,
    #[serde(default)]
    pub token_type: String,
    #[serde(default)]
    pub token_size: i32,
    #[serde(default)]
    pub addr_time_patterns: Vec<AddressPatterns>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
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


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GlobalConstraint {
    pub max_width: i32,
    pub max_height: i32,
    pub max_latency: i32,
    pub max_period: i32,
    pub max_energy: i32,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlimpLibrary {
    pub entries: Vec<AlimpEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlimpEntry {
    pub func: String,
    pub instances: Vec<AlimpInstance>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
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
    pub input_addr_time_patterns: Vec<AddressPatterns>,
    #[serde(default)]
    pub output_addr_time_patterns: Vec<AddressPatterns>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HyperParameter {
    pub bind_w_area: i32,
    pub bind_w_energy: i32,
    pub bind_w_latency: i32,
    pub bind_relaxation_factor: f64,
    pub place_relaxation_factor: f64,
    pub place_reserved_routing_size: i32,
}
 

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TechConstraint {
    pub width_grid: f64, // um
    pub height_grid: f64, // um
    pub width_drra: f64, // um
    pub height_drra: f64, // um
    pub grid_per_drra_width: i32, // number of grid blocks for one DRRA cell's width 
    pub grid_per_drra_height: i32, // number of grid blocks for one DRRA cell's height
    pub clock_frequency: f64, // Hz
    pub required_slew: f64,
    pub initial_slew: f64,
    pub buffer_slew_declined_factor: f64,
    pub buffer_delay_improved_factor: f64,
    pub register_slew_constant: f64,
    pub slew_rates: Vec<f64>,
    pub timing_table: Vec<TimingRow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimingRow {
    pub rows: Vec<f64>,
}
 

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoutingGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub channels: Vec<Channel>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub weight: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub weight: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub app_edge_id: String,
    pub source: Vec<String>,
    pub target: Vec<String>,
    pub traffic: f64,
    pub path: Vec<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloorPlan {
    pub app_node_ids: Vec<String>,
    pub app_edge_ids: Vec<String>,
    pub max_width: i32,
    pub max_height: i32,
    pub pos: Vec<RectanglePosition>,
    pub shape: Vec<RectangleShape>,
    pub source_node: Vec<u32>,
    pub target_node: Vec<u32>,
    pub source_port: Vec<u32>,
    pub target_port: Vec<u32>,
    pub top_space: Vec<u32>,
    pub bottom_space: Vec<u32>,
    pub left_space: Vec<u32>,
    pub right_space: Vec<u32>,
    pub conn: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RectanglePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RectangleShape {
    pub width: i32,
    pub height: i32,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
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


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SynthesizedInformation {
    pub alimp_bindings: Vec<AlimpBinding>,
    pub placements: Vec<Placement>,
    pub max_width: i32,
    pub max_height: i32,
    pub max_latency: i32,
    pub routing_paths: Vec<RoutingPath>,
    pub wire_assignment: HashMap<String, String>,
    pub node_fire_times: HashMap<String, i32>,
    pub channel_width: HashMap<String, i32>,
    pub address_translations: Vec<AddressTranslation>,
    pub memory_synthesis: Vec<MemorySynthesis>,
    pub transporter_tables: Vec<TransporterTable>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub app_node_id: String,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoutingPath {
    pub app_edge_id: String,
    pub path: Vec<Coordinate>,
    pub delay: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Coordinate {
    pub x: u32,
    pub y: u32,
    pub port: u32, // 0: normal, 1: output, 2: input
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddressTranslation {
    pub app_node_id: String,
    pub port_id: String,
    pub address_assignment: HashMap<i32, (i32, i32, i32)>, // virtual address -> (from channel, bank index, physical address)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MemorySynthesis {
    pub app_node_id: String,
    pub port_id: String,
    pub memory_direction: String,
    pub memory_structure: Vec<MemoryStructure>, 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStructure {
    pub memory_type: String,
    pub memory_size: u32,
    pub input_channels: Vec<u32>,
    pub output_channels: Vec<u32>,
    pub corresponding_channels: Vec<u32>,
    pub placement: MemoryPlacement, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransporterISAFunc {
    Add,
    Sub,
    Mul,
    Div,
    ShiftRight,
    LogicShiftLeft,
    ArithmeticShiftLeft,
    And,
    Or,
    Xor,
    Not,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransporterISA {
    OCCUPIED,
    NOP { immediate: i32 },
    NOPR { r0: u32, immediate: i32 },
    LDI { r0: u32, immediate: i32 },
    BRN { r0: u32, immediate: i32 },
    MOVC { r2: u32, r1: u32, r0: u32, immediate: i32 },
    MOV { r1: u32, r0: u32, immediate: i32 },
    CAL { r2: u32, r1: u32, r0: u32, function: TransporterISAFunc },
    CALI { r2: u32, r1: u32, r0: u32, function: TransporterISAFunc },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransporterTable {
    pub transporter_id: String,
    pub fire_time: i32,
    pub end_time: i32,
    pub latency: u32,
    pub from: u32,
    pub to: u32,
    pub entries: Vec<TransportTableEntry>,
    pub ir: BTreeMap<i32, TransporterISA>, // time index -> instruction
    pub binary: Vec<u16>, // final instruction code in binary
    pub size: u32, // code size 
    pub placement: MemoryPlacement,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransportTableEntry {
    pub relative_time: i32,
    pub source_address: i32,
    pub target_address: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MemoryPlacement {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddressPatterns {
    #[serde(default)]
    pub address: i32,
    #[serde(default)]
    pub channel: i32,
    #[serde(default)]
    pub time: i32,
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
                width_grid: 0.0,
                height_grid: 0.0,
                width_drra: 0.0,
                height_drra: 0.0,
                grid_per_drra_width: 0,
                grid_per_drra_height: 0,
                clock_frequency: 0.0,
                required_slew: 0.0,
                initial_slew: 0.0,
                buffer_slew_declined_factor: 0.0,
                buffer_delay_improved_factor: 0.0,
                register_slew_constant: 0.0,
                slew_rates: vec![],
                timing_table: vec![],
            },
            routing_graph: RoutingGraph {
                nodes: vec![],
                edges: vec![],
                channels: vec![],
            },
            floor_plan: FloorPlan {
                app_node_ids: vec![],
                app_edge_ids: vec![],
                max_width: 0,
                max_height: 0,
                pos: vec![],
                shape: vec![],
                source_node: vec![],
                target_node: vec![],
                source_port: vec![],
                target_port: vec![],
                top_space: vec![],
                bottom_space: vec![],
                left_space: vec![],
                right_space: vec![],
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
                max_latency: 0,
                routing_paths: vec![],
                wire_assignment: std::collections::HashMap::new(),
                node_fire_times: std::collections::HashMap::new(),
                channel_width: std::collections::HashMap::new(),
                address_translations: vec![],
                memory_synthesis: vec![],
                transporter_tables: vec![],
            },
        }
    }
}


fn fmt_reg(r: u32) -> String {
    format!("R{}", r)
}

impl fmt::Display for TransporterISAFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TransporterISAFunc::*;
        let s = match self {
            Add => "ADD",
            Sub => "SUB",
            Mul => "MUL",
            Div => "DIV",
            ShiftRight => "SHR",
            LogicShiftLeft => "LSL",
            ArithmeticShiftLeft => "ASL",
            And => "AND",
            Or => "OR",
            Xor => "XOR",
            Not => "NOT",
        };
        write!(f, "{s}")
    }
}

impl fmt::Display for TransporterISA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TransporterISA::*;

        match self {
            OCCUPIED {} => {
                write!(f, "OCCUPIED")
            }
            NOP { immediate } => {
                write!(f, "NOP {}", immediate)
            }
            NOPR { r0, immediate } => {
                write!(f, "NOPR {} {}", fmt_reg(*r0), immediate)
            }
            LDI { r0, immediate } => {
                write!(f, "LDI {} {}", fmt_reg(*r0), immediate)
            }
            BRN { r0, immediate } => {
                write!(f, "BRN {} {}", fmt_reg(*r0), immediate)
            }
            MOVC { r0, r1, r2, immediate } => {
                write!(
                    f,
                    "MOVC {} {} {} {}",
                    fmt_reg(*r2),
                    fmt_reg(*r1),
                    fmt_reg(*r0),
                    immediate
                )
            }
            MOV { r0, r1, immediate } => {
                write!(
                    f,
                    "MOV {} {} {}",
                    fmt_reg(*r1),
                    fmt_reg(*r0),
                    immediate
                )
            }
            CAL { r0, r1, r2, function } => {
                write!(
                    f,
                    "CAL {} {} {} {}",
                    fmt_reg(*r2),
                    fmt_reg(*r1),
                    fmt_reg(*r0),
                    function
                )
            }
            CALI { r0, r1, r2, function } => {
                write!(
                    f,
                    "CALI {} {} {} {}",
                    fmt_reg(*r2),
                    fmt_reg(*r1),
                    fmt_reg(*r0),
                    function
                )
            }
        }
    }
}







