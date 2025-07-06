use crate::models;
use crate::file_handler;
use crate::command_runner; 

use models::{
    MemoryList, 
    Memory,
    BufferList, 
    Buffer,
    AddressPatternList,
    TranslationTableList,
};

pub struct ProcessModule {
    name: String,
    path: String,
    command: String,
    in_buf_path: String, 
    in_mem_path: String,
    out_mem_path: String,
    out_buf_path: String,        
    in_buf: BufferList, 
    in_mem: MemoryList,
    out_buf: BufferList, 
    out_mem: MemoryList,
    is_there_input: bool,
    is_there_output: bool,
    address_pattern: AddressPatternList, 
    translation_table: TranslationTableList,
}

impl ProcessModule {
    pub fn new(name: String, path: String, in_nodes: Vec<String>, out_nodes: Vec<String>, cmd: String) -> Self {
        Self {
            name: name.clone(),
            path: path.clone(),
            command: cmd.clone(),
            in_buf_path: format!("{}/mem/{}_inBuf.json", path, name), 
            in_mem_path: format!("{}/mem/{}_inMem.json", path, name),
            out_mem_path: format!("{}/mem/{}_outMem.json", path, name),
            out_buf_path: format!("{}/mem/{}_outBuf.json", path, name),
            in_buf: BufferList { mem: vec![] },
            in_mem: MemoryList { line: vec![] },
            out_mem: MemoryList { line: vec![] },
            out_buf: BufferList { mem: vec![] }, 
            is_there_input: in_nodes.len() > 0,
            is_there_output: out_nodes.len() > 0,
            address_pattern: AddressPatternList { addr_ptrn: vec![] }, 
            translation_table: TranslationTableList { list: vec![] },
        }
    }


    fn handle_input_address_pattern(&mut self, global_time: i64) -> Result<(), Box<dyn std::error::Error>>  {
        println!("[@{}] Starting input Addr Translation.", global_time);
        let address_pattern_file = format!("{}/{}_inAP.json", self.path, self.name);
        self.address_pattern = file_handler::load_json_file(&address_pattern_file)?;
        let translation_table_file = format!("{}/{}_inTT.json", self.path, self.name);
        self.translation_table = file_handler::load_json_file(&translation_table_file)?;
        
        self.in_buf = file_handler::load_json_file(&self.in_buf_path)?;

        // Sort the in_buf by cycle (earliest first)
        self.in_buf.mem.sort_by(|a, b| a.cycle.cmp(&b.cycle));
        // Sort the address_pattern by cycle (earliest first)
        self.address_pattern.addr_ptrn.sort_by(|a, b| a.cycle.cmp(&b.cycle));

        let mut ref_time = global_time;
        
        for ptrn in &self.address_pattern.addr_ptrn {
            ref_time = global_time + ptrn.cycle;
            
            // Find the translated address of the current memory address
            let translated_address_lists: Vec<_> = self
                .translation_table
                .list
                .iter()
                .filter(|x| x.addr_in == ptrn.address)
                .collect();
            let translated_address: i64 = match translated_address_lists.len() {
                1 => translated_address_lists[0].addr_out.clone(),
                _ => panic!(
                        "[@{}] Fail to translate address for inAddr: 0x{:X}", 
                        ref_time, 
                        ptrn.address 
                    ),
            };
            
            // Get the elements that match the translated address and have cycle <= ref_time - 1
            let candidates: Vec<_> = self
                .in_buf
                .mem
                .iter()
                .enumerate()
                .filter(|(_, buf)| buf.address == translated_address && (buf.cycle + 1) <= ref_time)
                .collect();
            // must find only one element, otherwise it is an error
            match candidates.len() {
                1 => {
                    let (index, _) = candidates[0];
                    let removed_buf = self.in_buf.mem.remove(index);
                    let new_line = Memory {
                        address: ptrn.address,
                        value: removed_buf.value.clone(),
                    };
                    self.in_mem.line.push(new_line);
                }
                0 => {
                    panic!(
                        "[@{}] For inAddr 0x{:X} colud not find a value", 
                        ref_time, 
                        ptrn.address
                    );
                }
                _ => {
                    panic!(
                        "[@{}] For inAddr 0x{:X} write collision detected", 
                        ref_time, 
                        ptrn.address
                    );
                }   

            }
        } 

        file_handler::write_json_file(&self.in_mem_path, &self.in_mem)?;
        println!("[@{}] Input Addr Translation done.", ref_time);
        Ok(())
    }


    fn handle_output_address_pattern(&mut self, global_time: i64) -> Result<(), Box<dyn std::error::Error>> {
        println!("[@{}] Starting output Addr Translation.", global_time);
        let address_pattern_file = format!("{}/{}_outAP.json", self.path, self.name);
        self.address_pattern = file_handler::load_json_file(&address_pattern_file)?;
        let translation_table_file = format!("{}/{}_outTT.json", self.path, self.name);
        self.translation_table = file_handler::load_json_file(&translation_table_file)?;
    
        self.out_mem = file_handler::load_json_file(&self.out_mem_path)?;
        
        // Sort the address_pattern by cycle (earliest first)
        self.address_pattern.addr_ptrn.sort_by(|a, b| a.cycle.cmp(&b.cycle));

        let mut ref_time = global_time;
        
        for ptrn in &self.address_pattern.addr_ptrn {
            ref_time = global_time + ptrn.cycle;
            
            // Find the translated address of the current memory address
            let translated_address_lists: Vec<_> = self
                .translation_table
                .list
                .iter()
                .filter(|x| x.addr_in == ptrn.address)
                .collect();
            let translated_address: i64 = match translated_address_lists.len() {
                1 => translated_address_lists[0].addr_out.clone(),
                _ => panic!(
                        "[@{}] Fail to translate address for inAddr: 0x{:X}", 
                        ref_time, 
                        ptrn.address 
                    ),
            };
            
            // Get the element in the memory that is about to be transferred
            let value_lists: Vec<_> = self
                .out_mem
                .line
                .iter()
                .filter(|x| x.address == ptrn.address)
                .collect();
            // must find only one element, otherwise it is an error
            let value: String = match value_lists.len() {
                1 => value_lists[0].value.clone(),
                _ => panic!(
                        "[@{}] For inAddr 0x{:X} colud not find a value", 
                        ref_time, 
                        ptrn.address
                    ),
            };

            let new_line = Buffer {
                cycle: ref_time,
                address: translated_address,
                value: value.clone(),
            };
            self.out_buf.mem.push(new_line);
        } 

        file_handler::write_json_file(&self.out_buf_path, &self.out_buf)?;
        println!("[@{}] Output Addr Translation done.", ref_time);
        Ok(())
    }


    pub fn run(&mut self, global_time: i64) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_there_input {
            self.handle_input_address_pattern(global_time)?;
        }
        
        println!("Main process is preparing to run: {}", self.command);
        let output = command_runner::run_command(&self.command); 
        match output {
            Ok(x) => println!("Process output = {}", x.trim_end()),
            Err(e) => panic!("Process error = {:#?}", e),
        }
        
        if self.is_there_output {
            self.handle_output_address_pattern(global_time)?;
        }
        
        Ok(())
    }
}
