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

    #[arg(long = "input_image_channel", help="Input image channel")]
    input_image_channel: i32,

    #[arg(long = "input_image_size", help="Input image size")]
    input_image_size: i32,

    #[arg(long = "kernel_size", help="Kernel size")]
    kernel_size: i32,

    #[arg(long = "stride", help="Stride", default_value_t=1)]
    stride: i32,

    #[arg(long = "mode", help="Mode", default_value_t=String::from("average"))]
    mode: String,
}



fn pooling_2d(image: &Array2<f64>, kernel_size: usize, stride: usize, mode: &str) -> Array2<f64> {
    let (h, w) = image.dim();
    let kh = kernel_size;
    let kw = kernel_size;
    let oh = (h - kh) / stride + 1;
    let ow = (w - kw) / stride + 1;

    let mut pooled = Array2::<f64>::zeros((oh, ow));

    for i in 0..oh {
        for j in 0..ow {
            let h_start = i * stride;
            let w_start = j * stride;
            let patch = image.slice(s![h_start..h_start + kh, w_start..w_start + kw]);

            let value = match mode {
                "max" => patch.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
                "average" => patch.sum() / (kh * kw) as f64,
                _ => panic!("Unsupported pooling mode: {}", mode),
            };

            pooled[[i, j]] = value.tanh();
        }
    }

    pooled
}


fn pooling(
    image: &ArrayD<f64>, // shape: [channels, H, W]
    kernel_size: usize,
    stride: usize,
    mode: &str,
) -> Result<ArrayD<f64>, Box<dyn std::error::Error>> {
    let image = image.clone().into_dimensionality::<ndarray::Ix3>()?;
    let (channels, _, _) = image.dim();
    let mut pooled_channels = Vec::with_capacity(channels);

    for ch in 0..channels {
        let image_2d = image.index_axis(Axis(0), ch);
        let pooled_2d = pooling_2d(&image_2d.to_owned(), kernel_size, stride, mode);
        pooled_channels.push(pooled_2d);
    }

    let result = ndarray::stack(
        Axis(0),
        &pooled_channels.iter().map(|x| x.view()).collect::<Vec<_>>(),
    )?;

    Ok(result.into_dyn())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input: Vec<Vec<f64>> = util::json_to_fpmem(&args.in_mem)?; 
    let image: ArrayD<f64> = util::mem2mat(&input, &[args.input_image_channel as usize, args.input_image_size as usize, args.input_image_size as usize])?;
    let output_image = pooling(&image, args.kernel_size as usize, args.stride as usize, &args.mode)?;
    let output_fpmem = util::mat2mem(&output_image)?;
    let output_fpmem_vec: Vec<Vec<f64>> = output_fpmem.iter().map(|a| a.to_vec()).collect();
    util::fpmem_to_json(&output_fpmem_vec, &args.out_mem)?;

    Ok(())
}

