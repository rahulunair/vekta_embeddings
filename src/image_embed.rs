use anyhow::{Context, Result};
use fastembed::{ImageEmbedding, ImageEmbeddingModel, ImageInitOptions};
use image::GenericImageView;
use serde_json::json;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

mod utils;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_help();
        return Ok(());
    }

    let batch_size = utils::detect_system_resources();

    utils::log("Initializing image embedding model...");
    let model = ImageEmbedding::try_new(
        ImageInitOptions::new(ImageEmbeddingModel::ClipVitB32).with_show_download_progress(true),
    )?;
    utils::log("Model initialized successfully.");

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let image_paths: Vec<String> = stdin
        .lock()
        .lines()
        .map(|line| line.map(|l| l.trim().to_string()))
        .collect::<io::Result<_>>()?;

    let total_images = image_paths.len();
    utils::log(&format!("Processing {} images...", total_images));

    for (batch_index, batch) in image_paths.chunks(batch_size).enumerate() {
        utils::log(&format!(
            "Embedding batch {} of {}",
            batch_index + 1,
            (total_images + batch_size - 1) / batch_size
        ));
        let embeddings = model
            .embed(batch.to_vec(), None)
            .context("Failed to embed images")?;

        for (path, embedding) in batch.iter().zip(embeddings.iter()) {
            let metadata = get_image_metadata(path)?;
            let output = json!({
                "label": metadata.label,
                "vector": embedding,
                "metadata": metadata
            });
            writeln!(stdout, "{}", output).context("Failed to write output")?;
        }
    }

    utils::log(&format!("Processed {} images successfully.", total_images));
    Ok(())
}

#[derive(serde::Serialize)]
struct ImageMetadata {
    label: String,
    file_path: String,
    file_name: String,
    file_size: u64,
    image_format: String,
    dimensions: (u32, u32),
    color_space: String,
}

fn get_image_metadata(path: &str) -> Result<ImageMetadata> {
    let file_path = Path::new(path);
    let file_name = file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let file_size = fs::metadata(path)?.len();

    let img = image::open(path)?;
    let dimensions = img.dimensions();
    let color_space = match img.color() {
        image::ColorType::L8 => "Grayscale",
        image::ColorType::La8 => "GrayscaleAlpha",
        image::ColorType::Rgb8 => "RGB",
        image::ColorType::Rgba8 => "RGBA",
        _ => "Unknown",
    };

    let image_format = match image::guess_format(&fs::read(path)?) {
        Ok(format) => format!("{:?}", format),
        Err(_) => "Unknown".to_string(),
    };

    Ok(ImageMetadata {
        label: file_name.clone(),
        file_path: path.to_string(),
        file_name,
        file_size,
        image_format,
        dimensions,
        color_space: color_space.to_string(),
    })
}

fn print_help() {
    eprintln!("vie - Vekta Image Embedder");
    eprintln!("Usage: vie [-h|--help]");
    eprintln!();
    eprintln!("It reads image file paths from stdin, processes these images,");
    eprintln!("and outputs JSON-formatted embeddings with metadata to stdout.");
    eprintln!();
    eprintln!("The tool processes images in batches for efficiency.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help    Show this help message and exit");
    eprintln!();
    eprintln!("Example usage:");
    eprintln!("  find . -name '*.jpg' -o -name '*.png' | vie > image_embeddings.jsonl");
}
