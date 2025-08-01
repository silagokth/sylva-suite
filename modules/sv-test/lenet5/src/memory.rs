use sv_lib::sim::{MemoryList, Memory};
use sv_lib::file_handler::{write_json_file};
use clap::Parser;


// Arguments 
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "image", help="global memory image")]
    image: String,
    
    #[arg(long = "reference", help="global memory reference")]
    reference: String,
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut memory = load_image();
    let result = load_result();

    let mut output_image = MemoryList { line: Vec::new() };  
    for i in 0..memory.len() {
        output_image.line.push(Memory {
            address: i as i64,
            value: memory[i].clone(),
        }); 
    }

    memory.extend(result);
    let mut output_reference = MemoryList { line: Vec::new() };  
    for i in 0..memory.len() {
        output_reference.line.push(Memory {
            address: i as i64,
            value: memory[i].clone(),
        }); 
    }

    write_json_file(&args.image, &output_image)?;
    write_json_file(&args.reference, &output_reference)?;

    Ok(())
}


fn load_image() -> Vec<String> {
    vec![
        "fff500100001fffcfff800170009ffef0008000bfff40008000afff9000cffed".to_string(),
        "ffeaffea000a00120003ffeb000100060010fffefff7fff9ffff0007ffeeffeb".to_string(),
        "0006ffec0005ffe90009fff40009fff7fff3fff0fffafff7ffedfffcfff2fff8".to_string(),
        "fff9000afffefffd0014fff3000f00080008fffdffebfffc0005ffe9ffff000c".to_string(),
        "0017fff50012ffe9ffee0016ffed00000015ffe9fff0ffecffebffff00020010".to_string(),
        "fff400180016fffffffefff90003fffc000d000dffee000f0009000cfff8ffec".to_string(),
        "00070004fff1fffeffe9000dfffc0011ffec000500110005000bfffc0015fff7".to_string(),
        "fff10014fff00011ffe90018fff1ffff000b0015fff20009fffeffeefffc000c".to_string(),
        "0010fff2ffedfff7fffbffeefff2fff600000007001000100016fff90017000f".to_string(),
        "000e000000080010fffafff80017fffc0015fff800100002fff9fff70013ffef".to_string(),
        "000d001700100008fffafffeffeb0011fffc0014000affe9001100140016fffc".to_string(),
        "0016fff10008fff5000500140016fffa000bffef000bfff00009fffeffedffec".to_string(),
        "ffff000900090017fff3ffefffe8fffcfff7000b000b000b0001fffe0000fff8".to_string(),
        "fff6fff800140002fff1fff5ffe900010002fff0fff4fffd00110011fffdfffc".to_string(),
        "00180010fff0fff9ffff0001fff0fff30006000ffff5000affec0018fff0fff7".to_string(),
        "ffea0001ffff0000ffe8fffeffee0008fff900090010000a000dfffcffe9fffd".to_string(),
        "fffafff80014000fffe80005000c000f0008ffec0013ffe80014ffedfff60000".to_string(),
        "000d0002000100110008000dfff3ffee00000015fffefffc0004000bffeaffe9".to_string(),
        "001500060010fff8fffc0014ffebfffb0013ffebffeb0015fff6ffecfff80004".to_string(),
        "fff4ffeefffbfff90017fff20007ffeffff7fffc0003ffee0015fff6ffeafffe".to_string(),
        "ffe800140010ffed001000090014fff3ffe9fff1001200180013000dfff5000e".to_string(),
        "0008ffed000efff9ffe8fff9fff70015000afff6fffb0004fff1fff500010011".to_string(),
        "00050018fffeffeafffaffebfff4fffefffcfff100060000000f0013000f0000".to_string(),
        "0001ffe8fff8ffe90000000fffeffffafff800050000fffffff8000e0002fff4".to_string(),
        "000cfffffff8fff1ffe9000c00150000fff3fff6fffaffea000dfffdffe90009".to_string(),
        "000fffe8ffe9000e0009fff70001fff9ffef00070015fffc0009ffeb000afffb".to_string(),
        "ffee0002000ffff400030000fff0fffafff6ffe9fffdfff70009fffbfff80014".to_string(),
        "fff4000600080011001600000001ffe800140016fff6fff600070004fff1fffb".to_string(),
        "ffe9000a000bfff60016fffa0009fff7ffe8000300110012fff1ffe9000a000d".to_string(),
        "000c0004000c000ffff400170000ffebfff6000d0007ffeb000bfff000020011".to_string(),
        "fffe00100011000a00060011ffe900130012fff8fff0fff4fffd00120001fff5".to_string(),
        "ffed0005000d00080008fff6000700070001fffd0018fff8fffafff10000ffef".to_string(),
        "fff3ffecfff90002ffffffe8000600080009ffe90000fff3ffed0004fff2000d".to_string(),
        "fffe000afff5fffefff60000fffd0000ffeb00000007fffbfffdfffcfff7000d".to_string(),
        "000effecfff6000d0001fffbfffcfff40013000efffa00140010001100050001".to_string(),
        "000effedfff9000c0007fff7000000150015000fffedfff7fffe000b0018fff1".to_string(),
        "000f0009fff60006fff300000000000dffecfff40003ffe80001fff10002fffe".to_string(),
        "fff6fff40002000c000bfff700000010001800160004000b0012ffef00090000".to_string(),
        "ffebffec0008fff0fffc0007000fffea00150003fff0000dffec00030001fffa".to_string(),
        "ffea000e00000001000700070004001500080017fff7001800140000ffee000f".to_string(),
        "0014fff90009ffeafff9ffe80005ffedfff100050011000bffea001200050009".to_string(),
        "000afff700120004fff0fff0fffdfffcffe90007fffa00110011fff200030016".to_string(),
        "ffec0013fffa001600050000fffd00150006fff00001000a0003fff70017fff3".to_string(),
        "000cfff2fffdffec0016fffdffee000800140014fffaffeeffeeffeafffc0016".to_string(),
        "ffef000d0012000f0001ffedfff7000fffea0003ffff00070011ffe8fffd0015".to_string(),
        "fffe000a000cfff2ffeeffecfffafff3fff90014fff5000f000ffff60000000f".to_string(),
        "001000120011fffaffec00180016ffe80008000ffff300030009fff60008fffb".to_string(),
        "fff20008fffa00160002ffeb0018ffef0002ffeefff9fffcfff10006ffffffeb".to_string(),
        "fffbffecfff9fff50009000700000000fff0ffeffffefffa0007fffffff8ffeb".to_string(),
        "0006fffdfffd000f000000090004fffcffe8fff3fff60002ffeefff8fff1ffea".to_string(),
        "0004001700140018ffec00080011fffbffeaffef0018fffc00000004000d0006".to_string(),
        "0004000ffff900150013000000100013fff0ffe9fff40000fff60010fffc0011".to_string(),
        "fff70002000fffe9000effff0010000f000affef001200040000fff30001000a".to_string(),
        "fffc0008ffe90006000a0016000200090005fffc000cfff0000e00080002000d".to_string(),
        "fff6fffa0018fff9ffe8ffecffeeffe8ffef0000000effeb000cfff8fff0ffeb".to_string(),
        "0000000cfffd0017ffee0000fffa0001000c0004fffffff2fff70002ffed000f".to_string(),
        "0017ffe8000e00000009fffcffee0002fff40012fff1000dfffa00110000000f".to_string(),
        "001700000017ffefffec0006000500000005ffef00000010ffe9ffedffe9fff1".to_string(),
        "ffec0008fff4fffd000f0000ffff000a0011000ffff90001000a0014ffee0014".to_string(),
        "fffe0016000c000900100003ffea000900150014000fffeb0007ffebfff20015".to_string(),
        "ffeefff0ffe90000ffea0014ffeaffe90007ffec00090005ffecffeb0007000f".to_string(),
        "ffe9ffff0017fffc00040001fff7fff3fff5fff600060016ffe9ffebfffafff0".to_string(),
        "fffdffefffe80002fffa0000000d000400180011ffef0007ffe80010000a0002".to_string(),
        "ffeafff8fffd00040001000bffecfff8000affeafff80000ffedfff4000cfffc".to_string()
    ]
}


fn load_result() -> Vec<String> {
    vec![
        "001200060006000b0003000c000e000b000a0009000000000000000000000000".to_string()
    ]
}
