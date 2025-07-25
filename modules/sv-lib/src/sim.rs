use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// NodeConfig and NodeConfigMap
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeConfig {
    pub is_transporter: bool,
    pub in_names: Vec<String>,
    pub out_names: Vec<String>,
    #[serde(default)]
    pub process_cmd: String,
    #[serde(default)]
    pub delay: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeConfigMap {
    pub config_map: HashMap<String, NodeConfig>,
}

// TimeTable and tt_node
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeTable {
    pub tt: Vec<TTNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TTNode {
    #[serde(default)]
    pub cycle: i64,
    pub node_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransporterInstructionList {
    pub inst_list: Vec<TransporterInstruction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransporterInstruction {
    #[serde(default)]
    pub cycle: i64,
    #[serde(default)]
    pub addr_rd: i64,
    #[serde(default)]
    pub addr_wr: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct MemoryList {
    pub line: Vec<Memory>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Memory {
    #[serde(default)]
    pub address: i64,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BufferList {
    pub mem: Vec<Buffer>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Buffer {
    #[serde(default)]
    pub cycle: i64,
    #[serde(default)]
    pub address: i64,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressPatternList {
    pub addr_ptrn: Vec<AddressPattern>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddressPattern {
    #[serde(default)]
    pub address: i64,
    #[serde(default)]
    pub cycle: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslationTableList {
    pub list: Vec<TranslationTable>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranslationTable {
    #[serde(default)]
    pub addr_in: i64,
    #[serde(default)]
    pub addr_out: i64,
}

