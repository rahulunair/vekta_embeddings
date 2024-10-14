use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use serde_json::json;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

mod utils;

const CHUNK_SIZE: usize = 256;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_help();
        return Ok(());
    }

    let batch_size = utils::detect_system_resources();

    utils::log("Initializing text embedding model...");
    let model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::AllMiniLML6V2Q).with_show_download_progress(true),
    )?;
    utils::log("Model initialized successfully.");

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut file_count = 0;
    for line in stdin.lock().lines() {
        let path = line.context("Failed to read input line")?;
        let path = path.trim();

        utils::log(&format!("Processing file: {}", path));
        let content =
            fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path))?;

        let chunks = split_into_chunks(&content, CHUNK_SIZE);

        for (batch_index, batch) in chunks.chunks(batch_size).enumerate() {
            utils::log(&format!(
                "  Embedding batch {} of file {}",
                batch_index + 1,
                path
            ));
            let embeddings = model.embed(batch.to_vec(), None).with_context(|| {
                format!(
                    "Failed to embed batch {} of file: {}",
                    batch_index + 1,
                    path
                )
            })?;

            for (i, embedding) in embeddings.iter().enumerate() {
                let chunk_index = i + (batch_index * batch_size);
                let (start_line, end_line) = get_line_range(&content, chunk_index, CHUNK_SIZE);
                let metadata = get_file_metadata(path, chunk_index, start_line, end_line);

                let output = json!({
                    "label": metadata.label,
                    "vector": embedding,
                    "metadata": metadata
                });
                writeln!(stdout, "{}", output).context("Failed to write output")?;
            }
        }
        file_count += 1;
    }

    utils::log(&format!("Processed {} files successfully.", file_count));
    Ok(())
}

#[derive(serde::Serialize)]
struct FileMetadata {
    label: String,
    file_path: String,
    file_name: String,
    chunk_index: usize,
    start_line: usize,
    end_line: usize,
    content_preview: String,
}

fn get_file_metadata(
    path: &str,
    chunk_index: usize,
    start_line: usize,
    end_line: usize,
) -> FileMetadata {
    let file_path = Path::new(path);
    let file_name = file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let content = fs::read_to_string(path).unwrap_or_default();
    let content_preview = content
        .lines()
        .skip(start_line)
        .take(end_line - start_line)
        .collect::<Vec<_>>()
        .join("\n");

    FileMetadata {
        label: format!("{}_part{}", file_name, chunk_index),
        file_path: path.to_string(),
        file_name,
        chunk_index,
        start_line,
        end_line,
        content_preview: content_preview.chars().take(100).collect::<String>() + "...",
    }
}

fn get_line_range(content: &str, chunk_index: usize, chunk_size: usize) -> (usize, usize) {
    let start_word = chunk_index * chunk_size;
    let end_word = (chunk_index + 1) * chunk_size;

    let mut line_count = 0;
    let mut word_count = 0;
    let mut start_line = 0;
    let mut end_line = 0;

    for line in content.lines() {
        let words_in_line = line.split_whitespace().count();
        if word_count < start_word {
            start_line = line_count;
        }
        if word_count < end_word {
            end_line = line_count;
        } else {
            break;
        }
        word_count += words_in_line;
        line_count += 1;
    }

    (start_line, end_line + 1)
}

fn split_into_chunks(text: &str, chunk_size: usize) -> Vec<String> {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .chunks(chunk_size)
        .map(|chunk| chunk.join(" "))
        .collect()
}

fn print_help() {
    eprintln!("vte - Vekta Text Embedder");
    eprintln!("Usage: vte [-h|--help]");
    eprintln!();
    eprintln!("It reads file paths from stdin, processes the text in these files,");
    eprintln!("and outputs JSON-formatted embeddings to stdout.");
    eprintln!();
    eprintln!("The tool splits text into chunks and processes them in batches for efficiency.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help    Show this help message and exit");
    eprintln!();

    eprintln!("Example usage:");
    eprintln!("  find . -name '*.txt' | vte > text_embeddings.jsonl");
}
