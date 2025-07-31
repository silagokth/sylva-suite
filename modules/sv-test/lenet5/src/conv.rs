use clap::Parser;
use ndarray::{s, Array2, ArrayD, Axis};

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

    #[arg(long = "kernel", help="Kernel file")]
    kernel: String,

    #[arg(long = "bias", help="Bias file")]
    bias: String,

    #[arg(long = "input_image_channel", help="Input image channel")]
    input_image_channel: i32,

    #[arg(long = "input_image_size", help="Input image size")]
    input_image_size: i32,

    #[arg(long = "kernel_channel", help="Kernel channel")]
    kernel_channel: i32,

    #[arg(long = "kernel_size", help="Kernel size")]
    kernel_size: i32,

    #[arg(long = "stride", help="Stride", default_value_t=1)]
    stride: i32,
}



fn conv_2d(image: &Array2<f64>, kernel: &Array2<f64>, stride: usize) -> Array2<f64> {
    let (h, w) = image.dim();
    let (kh, kw) = kernel.dim();
    let oh = (h - kh) / stride + 1;
    let ow = (w - kw) / stride + 1;

    let mut conv_image = Array2::<f64>::zeros((oh, ow));

    for i in 0..oh {
        for j in 0..ow {
            let h_start = i * stride;
            let w_start = j * stride;

            let patch = image.slice(s![h_start..h_start+kh, w_start..w_start+kw]);
            let sum = patch.iter()
                .zip(kernel.iter())
                .map(|(a, b)| a * b)
                .sum::<f64>();
            conv_image[[i, j]] = sum.tanh();
        }
    }

    conv_image
}



fn conv(
    image: &ArrayD<f64>,     // shape: [in_channels, H, W]
    kernel: &ArrayD<f64>,    // shape: [out_channels, KH, KW]
    bias: &ArrayD<f64>,      // shape: [out_channels, H, W]
    stride: usize
) -> Result<ArrayD<f64>, Box<dyn std::error::Error>> {
    let image = image.clone().into_dimensionality::<ndarray::Ix3>()?;
    let kernel = kernel.clone().into_dimensionality::<ndarray::Ix3>()?;
    let bias = bias.clone().into_dimensionality::<ndarray::Ix3>()?;

    let (out_channels, kh, kw) = kernel.dim();
    let (in_channels, h, w) = image.dim();

    let oh = (h - kh) / stride + 1;
    let ow = (w - kw) / stride + 1;

    // Validate bias shape
    let (bias_oc, bias_oh, bias_ow) = bias.dim();
    assert_eq!(bias_oc, out_channels);
    assert_eq!(bias_oh, oh);
    assert_eq!(bias_ow, ow);

    let mut output_vec = Vec::new();
    
    for out_ch in 0..out_channels {
        let mut out_map = None;

        for in_ch in 0..in_channels {
            let input_2d = image.index_axis(Axis(0), in_ch);
            let kernel_2d = kernel.index_axis(Axis(0), out_ch);

            let conv_result = conv_2d(&input_2d.to_owned(), &kernel_2d.to_owned(), stride);
            
            out_map = Some(match out_map {
                Some(existing) => existing + &conv_result,
                None => conv_result,
            });
        }

        // Add bias and collect the result
        let mut final_map = out_map.unwrap();
        let bias_2d = bias.index_axis(Axis(0), out_ch);
        final_map += &bias_2d;
        output_vec.push(final_map);
    }
    
    // Stack along new first axis (output channels)
    let stacked = ndarray::stack(
        Axis(0),
        &output_vec.iter().map(|x| x.view()).collect::<Vec<_>>()
    )?;

    Ok(stacked.into_dyn())
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut input: Vec<Vec<f64>> = util::json_to_fpmem(&args.in_mem)?; 
    let image: ArrayD<f64> = util::mem2mat(&input, &[args.input_image_channel as usize, args.input_image_size as usize, args.input_image_size as usize])?;

    input = util::json_to_fpmem(&args.kernel)?;
    let kernel: ArrayD<f64> = util::mem2mat(&input, &[args.kernel_channel as usize, args.kernel_size as usize, args.kernel_size as usize])?;

    input = util::json_to_fpmem(&args.bias)?;
    let output_image_size = (args.input_image_size - args.kernel_size) / args.stride + 1;
    let bias: ArrayD<f64> = util::mem2mat(&input, &[args.kernel_channel as usize, output_image_size as usize, output_image_size as usize])?;

    let output_image = conv(&image, &kernel, &bias, args.stride as usize)?;
    let output_fpmem = util::mat2mem(&output_image)?;
    let output_fpmem_vec: Vec<Vec<f64>> = output_fpmem.iter().map(|a| a.to_vec()).collect();
    util::fpmem_to_json(&output_fpmem_vec, &args.out_mem)?;

    Ok(())
}

