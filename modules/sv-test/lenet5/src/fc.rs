use clap::Parser;
use ndarray::{ArrayD, Axis};

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

    #[arg(long = "input_size", help="Input size")]
    input_size: i32,

    #[arg(long = "output_size", help="Output size")]
    output_size: i32,

    #[arg(long = "activation", help="Activation", default_value_t=String::from("tanh"))]
    activation: String,

    #[arg(long = "weight", help="Weight file")]
    weight: String,

    #[arg(long = "bias", help="Bias file")]
    bias: String,
}




fn fc(
    input_vector: &ArrayD<f64>,       // shape: [1, in_features]
    weight_matrix: &ArrayD<f64>,      // shape: [out_features, in_features]
    bias_vector: &ArrayD<f64>,        // shape: [1, out_features]
    activation: &str,                 // "softmax", "tanh", or ""
) -> Result<ArrayD<f64>, Box<dyn std::error::Error>> {
    let input = input_vector.clone().into_dimensionality::<ndarray::Ix2>()?;
    let weights = weight_matrix.clone().into_dimensionality::<ndarray::Ix2>()?;
    let bias = bias_vector.clone().into_dimensionality::<ndarray::Ix2>()?;

    // Matrix-vector multiplication: y = W * x
    let output = input.dot(&weights.t()) + &bias;

    // Apply activation function
    let activated = match activation {
        "softmax" => {
            let row = output.index_axis(Axis(0), 0);
            let max = row.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let exps: Vec<f64> = row.iter().map(|&x| (x - max).exp()).collect();
            let sum: f64 = exps.iter().sum();
            let softmaxed = exps.into_iter().map(|x| x / sum).collect::<Vec<f64>>();
            Ok(ndarray::Array2::from_shape_vec((1, softmaxed.len()), softmaxed)?.into_dyn())
        }
        "tanh" => Ok(output.mapv(|x| x.tanh()).into_dyn()),
        "" => Ok(output.into_dyn()),
        _ => Err(format!("Unsupported activation: {}", activation).into())
    };

    activated
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut input: Vec<Vec<f64>> = util::json_to_fpmem(&args.in_mem)?; 
    let image: ArrayD<f64> = util::mem2mat(&input, &[1, args.input_size as usize])?;

    input = util::json_to_fpmem(&args.weight)?;
    let weight_matrix: ArrayD<f64> = util::mem2mat(&input, &[args.output_size as usize, args.input_size as usize])?;

    input = util::json_to_fpmem(&args.bias)?;
    let bias_vector: ArrayD<f64> = util::mem2mat(&input, &[1, args.output_size as usize])?;

    let output_image = fc(&image, &weight_matrix, &bias_vector, &args.activation)?;
    let output_fpmem = util::mat2mem(&output_image)?;
    let output_fpmem_vec: Vec<Vec<f64>> = output_fpmem.iter().map(|a| a.to_vec()).collect();
    util::fpmem_to_json(&output_fpmem_vec, &args.out_mem)?;

    Ok(())
}

