use sv_lib::model::{DataBase, AddressTranslation, TranslationTable, TLBImplementation}; 
use sv_lib::file_handler;
use log::{info, error};
use std::collections::{HashMap};


fn apply_agu(
    physical_addrs: &Vec<i32>,
) -> Result<TLBImplementation, &'static str> {
    if physical_addrs.is_empty() {
        return Err("Empty address list");
    }

    if physical_addrs.len() == 1 {
        return Ok(TLBImplementation::AGU {
            value: physical_addrs[0] as u32,
            i: 1,
            j: 1,
            k: 1,
            stride_i: 0,
            stride_j: 0,
            stride_k: 0,
        });
    }


    let base = physical_addrs[0];
    let stride_i = physical_addrs[1] - physical_addrs[0];

    // -------- detect i --------
    let mut count_i = 1;
    while count_i < physical_addrs.len() {
        let expected = base + (count_i as i32) * stride_i;
        if physical_addrs[count_i] != expected {
            break;
        }
        count_i += 1;
    }

    // Pure 1D
    if count_i == physical_addrs.len() {
        return Ok(TLBImplementation::AGU {
            value: base as u32,
            i: count_i as u32,
            j: 1,
            k: 1,
            stride_i: stride_i as i32,
            stride_j: 0,
            stride_k: 0,
        });
    }

    // -------- detect j --------
    let stride_j = physical_addrs[count_i] - base;

    let mut idx = 0;
    while idx < physical_addrs.len() {
        let i = idx % count_i;
        let j = idx / count_i;

        let expected = base 
            + (i as i32) * stride_i 
            + (j as i32) * stride_j;

        idx += 1;
        if physical_addrs[idx-1] != expected {
            break;
        }
    }
    if (idx % count_i) == 1 {
        // proceed to find 3D pattern
        if idx == physical_addrs.len() {
            return Err("Not affine 2D pattern");
        }
    }
    else if (idx % count_i) != 0 {
        return Err("Not affine 2D pattern");
    }
    let count_j = idx / count_i;

    if count_i * count_j == physical_addrs.len() {
        return Ok(TLBImplementation::AGU {
            value: base as u32,
            i: count_i as u32,
            j: count_j as u32,
            k: 1,
            stride_i: stride_i as i32,
            stride_j: stride_j as i32,
            stride_k: 0,
        });
    }

    // -------- detect k --------
    let stride_k = physical_addrs[count_i * count_j] - base;

    let mut idx = 0;
    while idx < physical_addrs.len() {
        let i = idx % count_i;
        let j = (idx / count_i) % count_j;
        let k = idx / (count_i * count_j);

        let expected = base
                + (i as i32) * stride_i
                + (j as i32) * stride_j
                + (k as i32) * stride_k;

        idx += 1;
        if physical_addrs[idx-1] != expected {
            return Err("Not affine 3D pattern");
        }
    }
    if (idx % (count_i * count_j)) != 0 {
        return Err("Not affine 3D pattern");
    }
    let count_k = physical_addrs.len() / (count_i * count_j);

    Ok(TLBImplementation::AGU {
        value: base as u32,
        i: count_i as u32, 
        j: count_j as u32, 
        k: count_k as u32,
        stride_i: stride_i as i32, 
        stride_j: stride_j as i32, 
        stride_k: stride_k as i32,
    })
}


fn apply_buffer(
    virtual_addrs: &Vec<i32>,
    physical_addrs: &Vec<i32>,
) -> Result<TLBImplementation, &'static str> {
    if virtual_addrs.is_empty() || physical_addrs.is_empty() {
        return Err("Empty address list");
    }

    if virtual_addrs.len() != physical_addrs.len() {
        return Err("Invalid address lists");
    }

    if virtual_addrs.iter().any(|&v| v < 0) || physical_addrs.iter().any(|&p| p < 0) {
        return Err("Addresses must be non-negative");
    }

    let total_length = physical_addrs.len();
    let offset = virtual_addrs[0];
    
    let new_virtual_addrs: Vec<i32> =
        virtual_addrs.iter().map(|&a| a - offset).collect();

    if new_virtual_addrs.iter().any(|&v| v < 0) {
        return Err("Virtual addresses must be >= base offset");
    }

    let mut buffer_size: usize = 1;

    loop {        
        let mut buffer = vec![0i32; buffer_size];
        let mut valid = true;

        for i in 0..total_length {
            let v = new_virtual_addrs[i] as usize;
            if v >= buffer_size {
                break;
            }
            buffer[v] = physical_addrs[i];
        }

        // Verify mapping
        for i in 0..total_length {
            let v = new_virtual_addrs[i] as usize;
            if buffer[v % buffer_size] != physical_addrs[i] {
                valid = false;
                break;
            }
        }

        if valid {
            let buffer_u32: Vec<u32> = buffer.iter().map(|&x| x as u32).collect();

            return Ok(TLBImplementation::TLB {
                size: buffer_size as u32,
                offset: offset as u32,
                map: buffer_u32,
            });
        }

        buffer_size <<= 1;

        if buffer_size > (1 << 12) {
            return Err("Buffer size exploded; mapping likely sparse or invalid");
        }
    }
}


fn build_translation_table(
    address_translation: &mut AddressTranslation,
) -> Result<(), Box<dyn std::error::Error>> {
    
    let assignment = &address_translation.address_assignment;

    // channel -> (virtual_addrs, physical_addrs)
    let mut per_channel: HashMap<i32, (Vec<i32>, Vec<i32>)> = HashMap::new();

    // group by channel
    for (&virtual_addr, &(channel_id, _bank_idx, physical_addr)) in assignment.iter() {
        let entry = per_channel
            .entry(channel_id)
            .or_insert_with(|| (Vec::new(), Vec::new()));

        entry.0.push(virtual_addr);
        entry.1.push(physical_addr);
    }

    // sort by virtual address
    for (_channel_id, (vaddrs, paddrs)) in per_channel.iter_mut() {
        let mut pairs: Vec<(i32, i32)> = vaddrs
            .iter()
            .copied()
            .zip(paddrs.iter().copied())
            .collect();

        pairs.sort_by_key(|(v, _)| *v);

        vaddrs.clear();
        paddrs.clear();
        for (v, p) in pairs {
            vaddrs.push(v);
            paddrs.push(p);
        }
    }

    // implementation style per channel
    for (channel_id, (virtual_addrs, physical_addrs)) in per_channel {
        let agu_impl = apply_agu(&physical_addrs);
        let buf_impl = apply_buffer(&virtual_addrs, &physical_addrs);

        let chosen_impl: TLBImplementation = match (&buf_impl, &agu_impl) {
            // Prefer small buffer
            (Ok(TLBImplementation::TLB { size, .. }), _) if *size <= 8 => {
                buf_impl.unwrap()
            }

            // Otherwise AGU
            (_, Ok(_)) => agu_impl.unwrap(),

            // Otherwise buffer
            (Ok(_), Err(_)) => buf_impl.unwrap(),

            // Neither works
            (Err(e1), Err(_e2)) => {
                return Err(format!(
                    "Failed to generate translation for channel {}: {}",
                    channel_id, e1
                )
                .into());
            }
        };

        address_translation.translation_table.insert(
            channel_id,
            TranslationTable {
                implementation: chosen_impl,
                program_code: Vec::new(), 
                program_addr: Vec::new(),
            },
        );
    }

    Ok(())
}


pub fn generate_code(
    address_translation: &mut AddressTranslation,
) -> Result<(), Box<dyn std::error::Error>> {

    fn fits_u16(v: u32) -> bool {
        v <= u16::MAX as u32
    }
    
    fn fits_i16(v: i32) -> bool {
        v >= i16::MIN as i32 && v <= i16::MAX as i32
    }

    for (&col, table) in address_translation.translation_table.iter_mut() {
        if col < 0 || col > ((1 << 4) - 1) {
            return Err("Column exceeds 4-bit code field".into());
        }
    
        let mut addr: Vec<u32> = Vec::new();
        let mut code: Vec<u32> = Vec::new();

        match &table.implementation {
            TLBImplementation::AGU { value, i, j, k, stride_i, stride_j, stride_k } => {
                // validate unsigned fields
                for &v in [value, i, j, k].iter() {
                    if !fits_u16(*v) {
                        return Err("AGU unsigned field exceeds 16-bit unsigned range".into());
                    }
                }
            
                // validate signed strides
                for &s in [stride_i, stride_j, stride_k].iter() {
                    if !fits_i16(*s as i32) {
                        return Err("AGU stride exceeds 16-bit signed range".into());
                    }
                }
            
                addr.push(7 << 2);
                code.push(0); // reset 
                
                addr.push(0 << 2);
                code.push(*value & 0xFFFF);
                addr.push(1 << 2);
                code.push(*i & 0xFFFF);
                addr.push(2 << 2);
                code.push(*j & 0xFFFF);
                addr.push(3 << 2);
                code.push(*k & 0xFFFF);
                addr.push(4 << 2);
                code.push((*stride_i as i32 as u32) & 0xFFFF);
                addr.push(5 << 2);
                code.push((*stride_j as i32 as u32) & 0xFFFF);
                addr.push(6 << 2);
                code.push((*stride_k as i32 as u32) & 0xFFFF);
            
                addr.push(7 << 2);
                code.push(1); // activate 
            }
            TLBImplementation::TLB { size, offset, map } => {
                if !fits_u16(*size) || !fits_u16(*offset) {
                    return Err("TLB size/offset exceeds 16-bit unsigned range".into());
                }

                addr.push(1 << 15);
                code.push(*offset);

                for (i, &entry) in map.iter().enumerate().take(*size as usize) {
                    if !fits_u16(entry) {
                        return Err("TLB map entry exceeds 16-bit unsigned range".into());
                    }
                    if i >= (1 << 12) {
                        return Err("TLB size exceeds 12bit limit".into());
                    }
                    addr.push((i << 2) as u32);
                    code.push(entry & 0xFFFF);
                }
            }
        }

        // store code per channel if needed
        table.program_code = code.clone();
        table.program_addr = addr.clone();
    }

    Ok(())
}



fn dump_data(
    address_translation: &AddressTranslation,
    module_dir: &str,
    name: &str,
    print: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut formatted = String::new();

    formatted.push_str(&format!(
        "AppNode: {}\nPort: {}\n\n",
        address_translation.app_node_id,
        address_translation.port_id
    ));

    formatted.push_str("== Address Assignment ==\n");
    for (vaddr, (channel, bank, paddr)) in &address_translation.address_assignment {
        formatted.push_str(&format!(
            "  vaddr {} -> channel {}, bank {}, paddr {}\n",
            vaddr, channel, bank, paddr
        ));
    }

    formatted.push_str("\n== Translation Table ==\n");
    for (channel, table) in &address_translation.translation_table {
        formatted.push_str(&format!(
            "  Channel {}: {}\n",
            channel,
            table.implementation
        ));
    }

    let path = format!("{}/{}.txt", module_dir, name);
    file_handler::write_file(&path, formatted.clone())?;

    if print {
        println!("{}", formatted);
    }

    Ok(())
}



#[allow(unused_variables)]
pub fn run(
    db: &mut DataBase, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start: tlb assembler");
    let module_dir = format!("{}/tlb", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };
    
    let address_translations = &mut db.synthesized_information.address_translations;

    for address_translation in address_translations.iter_mut() {
        let name = format!("TLB_{}_{}", 
            address_translation.app_node_id, 
            address_translation.port_id);

        build_translation_table(address_translation)?;
        generate_code(address_translation)?;
        
        dump_data(
            &address_translation,
            &module_dir,
            &name,
            false, // print
        )?; 
                
        info!("complete TLB implementation for {}", name);
    }

    info!("Finish: tlb assembler");
    Ok(())
}






#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_agu_1d_linear() {
        let addrs = vec![100, 104, 108, 112, 116];

        match apply_agu(&addrs) {
            Ok(TLBImplementation::AGU { value, i, j, k, stride_i, stride_j, stride_k }) => {
                assert_eq!(value, 100);
                assert_eq!(i, 5);
                assert_eq!(j, 1);
                assert_eq!(k, 1);
                assert_eq!(stride_i, 4);
                assert_eq!(stride_j, 0);
                assert_eq!(stride_k, 0);
            }
            Err(e) => panic!("Expected 1D AGU pattern, got error: {e}"),
            _ => panic!("Expected 1D AGU pattern, got unexpected error"),
        }
    }

    #[test]
    fn apply_agu_2d_row_major() {
        // 2D: base + i*4 + j*16
        // i in [0..4], j in [0..3]
        let mut addrs = Vec::new();
        let base = 1000;
        let si = 16;
        let sj = 4;

        for j in 0..3 {
            for i in 0..4 {
                addrs.push(base + i * si + j * sj);
            }
        }

        match apply_agu(&addrs) {
            Ok(TLBImplementation::AGU { value, i, j, k, stride_i, stride_j, stride_k }) => {
                assert_eq!(value, base as u32);
                assert_eq!(i, 4);
                assert_eq!(j, 3);
                assert_eq!(k, 1);
                assert_eq!(stride_i, si);
                assert_eq!(stride_j, sj);
                assert_eq!(stride_k, 0);
            }
            Err(e) => panic!("Expected 1D AGU pattern, got error: {e}"),
            _ => panic!("Expected 1D AGU pattern, got unexpected error"),
        }
    }

    #[test]
    fn apply_agu_3d_row_major() {
        // 3D: base + i*4 + j*16 + k*64
        // i in [0..4], j in [0..3], k in [0..2]
        let mut addrs = Vec::new();
        let base = 2000;
        let si = 16;
        let sj = 4;
        let sk = 64;

        for k in 0..2 {
            for j in 0..3 {
                for i in 0..4 {
                    addrs.push(base + i * si + j * sj + k * sk);
                }
            }
        }

        println!("addrs = {:?}", addrs);
        match apply_agu(&addrs) {
            Ok(TLBImplementation::AGU { value, i, j, k, stride_i, stride_j, stride_k }) => {
                assert_eq!(value, base as u32);
                assert_eq!(i, 4);
                assert_eq!(j, 3);
                assert_eq!(k, 2);
                assert_eq!(stride_i, si);
                assert_eq!(stride_j, sj);
                assert_eq!(stride_k, sk);
            }
            Err(e) => panic!("Expected 1D AGU pattern, got error: {e}"),
            _ => panic!("Expected 1D AGU pattern, got unexpected error"),
        }
    }

    #[test]
    fn apply_agu_single_element() {
        let addrs = vec![42];

        match apply_agu(&addrs) {
            Ok(TLBImplementation::AGU { value, i, j, k, stride_i, stride_j, stride_k }) => {
                assert_eq!(value, 42);
                assert_eq!(i, 1);
                assert_eq!(j, 1);
                assert_eq!(k, 1);
                assert_eq!(stride_i, 0);
                assert_eq!(stride_j, 0);
                assert_eq!(stride_k, 0);
            }
            Err(e) => panic!("Expected 1D AGU pattern, got error: {e}"),
            _ => panic!("Expected 1D AGU pattern, got unexpected error"),
        }
    }

    #[test]
    fn apply_agu_non_affine_fails() {
        let addrs = vec![100, 104, 109, 113, 115]; // break in stride

        let res = apply_agu(&addrs);

        assert!(res.is_err());
    }

    #[test]
    fn apply_agu_2d_but_with_hole_should_fail() {
        // supposed to be base + i*4 + j*16, but one element is wrong
        let addrs = vec![
            1000, 1004, 1008, 1012, // j = 0
            1016, 1020, 9999, 1028, // j = 1 (broken)
        ];

        let res = apply_agu(&addrs);

        assert!(res.is_err());
    }

    #[test]
    fn apply_agu_3d_but_with_jump_should_fail() {
        let mut addrs = Vec::new();
        let base = 100;
        let si = 4;
        let sj = 16;
        let sk = 64;

        for k in 0..2 {
            for j in 0..2 {
                for i in 0..2 {
                    addrs.push(base + i * si + j * sj + k * sk);
                }
            }
        }

        // Inject a bad jump
        addrs[5] += 3;

        let res = apply_agu(&addrs);
        assert!(res.is_err());
    }


    #[test]
    fn apply_buffer_contiguous() {
        let virtual_addrs = vec![10, 11, 12, 13];
        let physical_addrs = vec![100, 104, 108, 112];
    
        let tlb = apply_buffer(&virtual_addrs, &physical_addrs).unwrap();
    
        match tlb {
            TLBImplementation::TLB { size, offset, map } => {
                assert_eq!(offset, 10);
                assert_eq!(size, 4);
                assert_eq!(map[0], 100);
                assert_eq!(map[1], 104);
                assert_eq!(map[2], 108);
                assert_eq!(map[3], 112);
            }
            _ => panic!("Expected TLB implementation"),
        }
    }

    #[test]
    fn apply_buffer_nonzero_base() {
        let virtual_addrs = vec![42, 43, 44];
        let physical_addrs = vec![1000, 1004, 1008];
    
        let tlb = apply_buffer(&virtual_addrs, &physical_addrs).unwrap();
    
        match tlb {
            TLBImplementation::TLB { offset, size, map} => {
                assert_eq!(offset, 42);
                assert_eq!(size, 4); 
                assert_eq!(map[0], 1000);
                assert_eq!(map[1], 1004);
                assert_eq!(map[2], 1008);
            }
            _ => panic!("Expected TLB"),
        }
    }

    #[test]
    fn apply_buffer_sparse_virtuals() {
        let virtual_addrs = vec![100, 104, 108];
        let physical_addrs = vec![2000, 3000, 4000];
    
        let tlb = apply_buffer(&virtual_addrs, &physical_addrs).unwrap();
    
        match tlb {
            TLBImplementation::TLB { offset, size, map } => {
                assert_eq!(offset, 100);
                assert_eq!(size, 16); // need room for offsets 0,4,8
                assert_eq!(map[0], 2000);
                assert_eq!(map[4], 3000);
                assert_eq!(map[8], 4000);
            }
            _ => panic!("Expected TLB"),
        }
    }

    #[test]
    fn apply_buffer_optimisation() {
        let virtual_addrs = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let physical_addrs = vec![0, 1, 0, 1, 0, 1, 0, 1];
    
        let tlb = apply_buffer(&virtual_addrs, &physical_addrs).unwrap();
    
        match tlb {
            TLBImplementation::TLB { offset, size, map } => {
                assert_eq!(offset, 1);
                assert_eq!(size, 2); 
                assert_eq!(map[0], 0);
                assert_eq!(map[1], 1);
            }
            _ => panic!("Expected TLB"),
        }
    }


    #[test]
    fn apply_buffer_single_entry() {
        let virtual_addrs = vec![7];
        let physical_addrs = vec![999];
    
        let tlb = apply_buffer(&virtual_addrs, &physical_addrs).unwrap();
    
        match tlb {
            TLBImplementation::TLB { offset, size, map} => {
                assert_eq!(offset, 7);
                assert_eq!(size, 1);
                assert_eq!(map[0], 999);
            }
            _ => panic!("Expected TLB"),
        }
    }

    #[test]
    fn apply_buffer_empty() {
        let virtual_addrs = vec![];
        let physical_addrs = vec![];
    
        let res = apply_buffer(&virtual_addrs, &physical_addrs);
        assert!(res.is_err());
    }

    #[test]
    fn apply_buffer_mismatched_lengths() {
        let virtual_addrs = vec![1, 2, 3];
        let physical_addrs = vec![10, 20];
    
        let res = apply_buffer(&virtual_addrs, &physical_addrs);
        assert!(res.is_err());
    }

    #[test]
    fn apply_buffer_negative_address() {
        let virtual_addrs = vec![0, -1, 2];
        let physical_addrs = vec![10, 20, 30];
    
        let res = apply_buffer(&virtual_addrs, &physical_addrs);
        assert!(res.is_err());
    }
}
