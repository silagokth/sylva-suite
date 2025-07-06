use crate::models;
use crate::file_handler;

use models::{
    TransporterInstructionList, 
    BufferList, 
    Buffer
};
use serde_json::from_str;

pub struct TransporterModule {
    name: String,
    path: String,
    in_name: String,
    out_name: String,
    delay: i64,
    in_buf_name: String,
    out_buf_name: String,
    instruction: TransporterInstructionList,
    in_buffer: BufferList,
    out_buffer: BufferList,
}


impl TransporterModule {
    pub fn new(name: String, path: String, in_name: String, out_name: String, delay: i64) -> Self {
        Self {
            name: name.clone(),
            path: path.clone(),
            in_name,
            out_name,
            delay,
            in_buf_name: format!("{}/mem/{}_inBuf.json", path, name),
            out_buf_name: format!("{}/mem/{}_outBuf.json", path, name),
            instruction: TransporterInstructionList { inst_list: vec![] },
            in_buffer: BufferList { mem: vec![] },
            out_buffer: BufferList { mem: vec![] }, 
        }
    }

    fn get_instruction(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let filename = format!("{}/{}_TransInst.json", &self.path, &self.name);
        let file_content = file_handler::load_file(&filename)?;
        let instruction_data: TransporterInstructionList = from_str(&file_content)?;
        self.instruction = instruction_data;
        Ok(())
    }

    fn get_in_buffer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let file_content = file_handler::load_file(&self.in_buf_name)?;
        let buffer_data: BufferList = from_str(&file_content)?;
        self.in_buffer = buffer_data;
        Ok(())
    }

    fn write_buffer(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.out_buffer)?;
        file_handler::write_file(&self.out_buf_name, &content)?;
        Ok(())
    }

    pub fn run(&mut self, global_time: i64) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "[@{}] Transferring data from {} to {}", 
            global_time, 
            &self.in_name, 
            &self.out_name
        );

        self.get_instruction()?;
        self.get_in_buffer()?;

        // Sort the in_buffer by cycle (earliest first)
        self.in_buffer.mem.sort_by(|a, b| a.cycle.cmp(&b.cycle));
        // Sort the instruction list by cycle (earliest first)
        self.instruction.inst_list.sort_by(|a, b| a.cycle.cmp(&b.cycle));
        
        for inst in &self.instruction.inst_list {
            let ref_time = global_time + inst.cycle;
            
            // Find elements that match the address and have cycle <= ref_time - 1
            let candidates: Vec<_> = self
                .in_buffer
                .mem
                .iter()
                .enumerate()
                .filter(|(_, buf)| buf.address == inst.addr_rd && (buf.cycle + 1) <= ref_time)
                .collect();
           
            // Must find only one element, otherwise it is an error
            match candidates.len() {
                1 => {
                    let (index, _) = candidates[0];
                    let removed_buf = self.in_buffer.mem.remove(index);
                    let new_line = Buffer {
                        cycle: ref_time + self.delay,
                        address: inst.addr_wr,
                        value: removed_buf.value,
                    };
                    self.out_buffer.mem.push(new_line);
                }
                0 => {
                    panic!(
                        "[@{}] For inAddr 0x{:X} could not find a value", 
                        ref_time, 
                        inst.addr_rd 
                    );
                }
                _ => {
                    panic!(
                        "[@{}] For inAddr 0x{:X} write collision detected", 
                        ref_time, 
                        inst.addr_rd
                    );
                }   
            }
        }

        self.write_buffer()?;
        println!("Transporter done.");
        Ok(())
    }
}
