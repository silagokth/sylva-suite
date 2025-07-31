use clap::Parser;
use ndarray::{ArrayD, IxDyn};

mod util;

// Arguments 
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "global-image", help="path to global memory image")]
    global_image: Option<String>,

    #[arg(long = "in-mem", help="path to input memory")]
    in_mem: String,
    
    #[arg(long = "out-mem", help="path to output memory")]
    out_mem: String,

    #[arg(long = "input_row", help="Input row")]
    input_row: i32,

    #[arg(long = "input_col", help="Input col")]
    input_col: i32,

    #[arg(long = "output_row", help="Output row")]
    output_row: i32,

    #[arg(long = "output_col", help="Output col")]
    output_col: i32,
}



fn reshape(
    input_matrix: &ArrayD<f64>,
    input_row: usize,
    input_col: usize,
    output_row: usize,
    output_col: usize,
) -> ArrayD<f64> {
    assert_eq!(input_row * input_col, output_row * output_col);
    assert_eq!(input_matrix.ndim(), 2);
    assert_eq!(input_matrix.shape(), &[input_row, input_col]);

    let mut output_matrix = ArrayD::<f64>::zeros(IxDyn(&[output_row, output_col]));

    for i in 0..output_row {
        for j in 0..output_col {
            let abs_idx = i * output_col + j;
            let input_i = abs_idx / input_col;
            let input_j = abs_idx % input_col;
            output_matrix[[i, j]] = input_matrix[[input_i, input_j]];
        }
    }

    output_matrix
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input: Vec<Vec<f64>> = util::json_to_fpmem(&args.in_mem)?; 
    let image: ArrayD<f64> = util::mem2mat(&input, &[args.input_row as usize, args.input_col as usize])?;
    let output_image = reshape(&image, args.input_row as usize, args.input_col as usize, args.output_row as usize, args.output_col as usize);
    let output_fpmem = util::mat2mem(&output_image)?;
    let output_fpmem_vec: Vec<Vec<f64>> = output_fpmem.iter().map(|a| a.to_vec()).collect();
    util::fpmem_to_json(&output_fpmem_vec, &args.out_mem)?;

    Ok(())
}

