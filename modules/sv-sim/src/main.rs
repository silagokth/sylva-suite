mod models;
mod file_handler;
mod node;
mod command_runner;
mod process_module;
mod transporter_module;

use models::{NodeConfigMap, TimeTable};
use node::Node;
use clap::Parser;
use serde_json::from_str;
use std::collections::HashMap;
 
/// Arguments to locate JSON files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory containing the files
    #[arg(long)]
    dir: String,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config_dir = format!("{}/config_map.json", args.dir);
    let time_table_dir = format!("{}/time_table.json", args.dir);
    
    let mut sim = Sim::new(
        config_dir,
        time_table_dir,
        args.dir,
    )?;

    sim.run()?;
    Ok(())
}


struct Sim {
    node_insts: HashMap<String, Node>,
    config_map: NodeConfigMap,  
    time_table: TimeTable,
}

impl Sim {
    fn new(config_dir: String, time_table_dir: String, path: String) -> Result<Self, Box<dyn std::error::Error>> {
        let config_file = file_handler::load_file(&config_dir)?;
        let config_data: NodeConfigMap = from_str(&config_file)?;
        let time_table_file = file_handler::load_file(&time_table_dir)?;
        let time_table_data: TimeTable = from_str(&time_table_file)?;
        
        let mut node_insts: HashMap<String, Node> = HashMap::new();
        for (name, node_config) in config_data.config_map.iter() {
            println!("Instantiating: {}", name);
            let node_instance = Node::new(
                name.clone(),
                node_config.clone(),
                path.clone(),
            );
            node_insts.insert(name.clone(), node_instance);
        }

        println!("Completed initalisation process.\n");
        Ok(Self {
            node_insts: node_insts,
            config_map: config_data,
            time_table: time_table_data,
        })
    }

    fn run(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut done_list: Vec<String> = vec![];
        let mut todo_list: Vec<String> = self.config_map.config_map.keys().cloned().collect();
        
        while todo_list.len() > 0 {
            let mut nodes_to_remove: Vec<String> = vec![];

            for name in &todo_list {
                let mut all_inputs_done = true;
                for in_name in &self.config_map.config_map.get(name).unwrap().in_names {
                    if !done_list.contains(&in_name) {
                        all_inputs_done = false;
                        break;
                    }
                }

                // fires the node if all inputs connected to this node are done
                if all_inputs_done {
                    // get instruction lists for scheduling this node
                    let inst_lst: Vec<_> = self
                        .time_table
                        .tt
                        .iter()
                        .filter(|x| x.node_name == *name)
                        .collect();
                    let schedule_time: i64 = match inst_lst.len() {
                        1 => inst_lst[0].cycle,
                        0 => panic!("Fail to find the timing schedule for {}.", name),
                        _ => panic!("Firing more than once is not yet supported ({})", name),
                    }; 

                    // run the simulation 
                    let global_time = schedule_time;
                    println!("[@{}] Triggering: {}", global_time, name);
                    if let Some(node) = self.node_insts.get_mut(name) {
                        node.run(global_time)?;
                    } else {
                        panic!("Node {} not found in node_insts.", name);
                    }

                    println!("Finishing: {}\n", name);
                    done_list.push(name.clone());
                    nodes_to_remove.push(name.clone()); 
                }
            }
            
            // Remove all nodes that fired from the todo_list
            todo_list.retain(|name| !nodes_to_remove.contains(name));

            if nodes_to_remove.len() == 0 {
                panic!("Fail to fire a node (conflict graph detected).");
            }
        }
        Ok(())
    }
}
