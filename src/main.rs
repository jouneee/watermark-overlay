use std::{path::PathBuf, process::Output};
use clap::Parser;
use image::{DynamicImage, GenericImage, imageops};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'w', long, value_name = "PATH", required = true)]
    watermark: PathBuf,

    #[arg(short = 'i', long, value_name = "PATH", required = true)]
    input: PathBuf,

    #[arg(short = 'o', long, value_name = "PATH", required = true)]
    output: PathBuf,

    #[arg(long, value_name = "SCALE", default_value_t = 1.0)]
    scale: f32,
}

fn main() {
    let args = Args::parse();

    if !args.watermark.exists() {
        eprintln!("Please include watermark image: {:?}", args.watermark);
        std::process::exit(1);
    }

    if !args.input.exists() || !args.input.is_dir() {
        eprintln!("Input dir does not exist or is not directory {:?}", args.input);
        std::process::exit(1);
    }

    let watermark = image::open(&args.watermark)
        .expect("Failed to open watermark image");

    for image in WalkDir::new(&args.input)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let input_path = image.path();

        if !["jpg", "jpeg", "png", "bmp", "gif"].contains(
            &input_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase()
                .as_str(),
        ) {
            continue;
        }

        let mut main_image = match image::open(input_path) {
            Ok(img) => img,
            Err(e) => {
                eprintln!("Error opening image {}: {}", input_path.display(), e);
                continue;
            }
        };

        let watermark_width = (main_image.width() as f32 * args.scale) as u32;
        let watermark_height = (watermark.height() as f32 * watermark_width as f32 / watermark.width() as f32) as u32;

        let resized_watermark = imageops::resize(
            &watermark,
            watermark_width,
            watermark_height,
            imageops::FilterType::Triangle,
        );

        let x = main_image.width().saturating_sub(resized_watermark.width()) as i64;
        let y = main_image.height().saturating_sub(resized_watermark.height()) as i64;

        imageops::overlay(&mut main_image, &resized_watermark, x, y);

        let output_path = args.output.join(input_path.file_name().unwrap()).with_extension("png");
        
        if let Err(e) = main_image.save(output_path) {
            eprintln!("Failed to output image: {}", e);
        }

    }

    println!("Watermarking complete!");
}
