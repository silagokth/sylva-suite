use sv_lib::sim::{NodeConfig, BufferList};
use sv_lib::file_handler;
use crate::command_runner; 
use crate::process_module;
use crate::transporter_module;

use process_module::ProcessModule;
use transporter_module::TransporterModule;

pub struct Node {
    name: String,
    config: NodeConfig,
    path: String,
    process: Option<ProcessModule>,
    transporter: Option<TransporterModule>,
}

impl Node {
    pub fn new(name: String, cfg: NodeConfig, path: String) -> Self {
        let mut config = cfg; 
        let mut process = None;
        let mut transporter = None;

        if !config.is_transporter {
            // add relative path to the command 
            config.process_cmd.push_str(&format!(
                    " --global-image {}/mem/global_mem_image.json",
                    path,
            ));
            // add input/output memory images
            if config.in_names.len() > 0 {
                config.process_cmd.push_str(&format!(
                        " --in-mem {}/mem/{}_inMem.json",
                        path,
                        name,
                ));   
            }
            if config.out_names.len() > 0 {
                config.process_cmd.push_str(&format!(
                        " --out-mem {}/mem/{}_outMem.json",
                        path,
                        name,
                ));   
            }
            // instantiate the process node
            process = Some(ProcessModule::new(
                name.clone(), 
                path.clone(), 
                config.in_names.clone(), 
                config.out_names.clone(), 
                config.process_cmd.clone(),
            ));
        } else {
            if config.in_names.len() != 1 || config.out_names.len() != 1 {
                panic!("Transporter has more or less than one input/output node");
            }
            // instantiate the transporter node
            transporter = Some(TransporterModule::new(
                name.clone(),
                path.clone(),
                config.in_names[0].clone(),
                config.out_names[0].clone(),
                config.delay,
            ));
        }

        Self {
            name: name.clone(),
            config: config,
            path: path.clone(),
            process: process,
            transporter: transporter,
        } 
    }
    
    /*
     * This function couples together all the memory buffers 
     * from the input transporters into the input memory buffer 
     * for this functional node.
     * 
     * Note: The inBuffers from the inputs must not have conflicts, 
     * which is guaranteed by the transporters.
     * (Addresses from each path have to fit in their own address space) 
     * */
    fn consolidate_input_buffer(&self) -> Result<(), Box<dyn std::error::Error>> { 
        if self.config.is_transporter || self.config.in_names.len() < 1 {
            panic!("Node name: {} with {} inputs (transporter status: {})",
                &self.name,
                &self.config.in_names.len(),
                &self.config.is_transporter,
            );
        }
        
        let mut in_buf = BufferList { mem: vec![] };
        
        // Each edge writes their own output buffer files and 
        // has an individual set of address space, 
        // so we can collect them without having to 
        // use the offset to shift addresses.
        for name in &self.config.in_names {
            let filename = format!("{}/mem/{}_outBuf.json", self.path, name); 
            let buf: BufferList = file_handler::load_json_file(&filename)?;
            for item in buf.mem {
                in_buf.mem.push(item.clone())
            }  
        }
        
        // write an input buffer
        let write_filename = format!("{}/mem/{}_inBuf.json", self.path, self.name); 
        file_handler::write_json_file(&write_filename, &in_buf)?;
        Ok(())
    }

    /* 
     * For distributing output buffer, all we need to do is to 
     * make copies of the file for every transporter 
     * this process sending data to. 
     * 
     * Physical addresses in the outBuf file are already designed 
     * for each transporter to realise its own address space. 
     */
    fn distribute_output_buffer(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.is_transporter || self.config.out_names.len() < 1 {
            panic!("Node name: {} with {} outputs (transporter status: {})",
                &self.name,
                &self.config.out_names.len(),
                &self.config.is_transporter,
            );
        }
        
        let filename = format!("{}/mem/{}_outBuf.json", self.path, self.name); 
        for name in &self.config.out_names {
            let write_filename = format!("{}/mem/{}_inBuf.json", self.path, name); 
            let command = format!("cp {} {}", filename, write_filename);
            command_runner::run_command(&command)?;  
        }   
        Ok(())
    }


    pub fn run(&mut self, global_time: i64) -> Result<(), Box<dyn std::error::Error>> {
        println!("[@{}] Starting.", global_time);

        if self.process.is_some() {
            if self.config.in_names.len() > 0 {
                self.consolidate_input_buffer()?;
            }

            if let Some(process) = &mut self.process {
                println!("[@{}] Trigger process module.", global_time);
                process.run(global_time)?; 
            } 


            if self.config.out_names.len() > 0 {
                self.distribute_output_buffer()?;
            }
        }

        if let Some(transporter) = &mut self.transporter {
            println!("[@{}] Trigger Transporter.", global_time);
            transporter.run(global_time)?; 
        }
        
        Ok(())
    }
}
