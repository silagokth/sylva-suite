use sv_lib::sim::{
    TransporterInstructionList, 
    BufferList, 
    Buffer
};
use sv_lib::file_handler;


pub struct TransporterModule {
    in_name: String,
    out_name: String,
    delay: i64,
    in_buf_name: String,
    out_buf_name: String,
    instruction_name: String,
    instruction: TransporterInstructionList,
    in_buffer: BufferList,
    out_buffer: BufferList,
}


impl TransporterModule {
    pub fn new(name: String, path: String, in_name: String, out_name: String, delay: i64) -> Self {
        Self {
            in_name,
            out_name,
            delay,
            in_buf_name: format!("{}/mem/{}_inBuf.json", path, name),
            out_buf_name: format!("{}/mem/{}_outBuf.json", path, name),
            instruction_name: format!("{}/{}_TransInst.json", path, name),
            instruction: TransporterInstructionList { inst_list: vec![] },
            in_buffer: BufferList { mem: vec![] },
            out_buffer: BufferList { mem: vec![] }, 
        }
    }
    
    pub fn run(&mut self, global_time: i64) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "[@{}] Transferring data from {} to {}", 
            global_time, 
            &self.in_name, 
            &self.out_name
        );

        self.instruction = file_handler::load_json_file(&self.instruction_name)?;
        self.in_buffer = file_handler::load_json_file(&self.in_buf_name)?;

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

        file_handler::write_json_file(&self.out_buf_name, &self.out_buffer)?;
        println!("Transporter done.");
        Ok(())
    }
}
