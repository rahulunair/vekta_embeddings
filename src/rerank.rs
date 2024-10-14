use anyhow::{Context, Result};
use fastembed::{RerankInitOptions, RerankerModel, TextRerank};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

mod utils;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        print_help();
        return Ok(());
    }

    let query = &args[1];
    utils::log("Initializing reranker model...");
    let model = TextRerank::try_new(
        RerankInitOptions::new(RerankerModel::JINARerankerV1TurboEn)
            .with_show_download_progress(true),
    )?;
    utils::log("Model initialized successfully.");

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let input: Vec<Value> = stdin
        .lock()
        .lines()
        .map(|line| serde_json::from_str(&line.unwrap()).unwrap())
        .collect();

    let documents: Vec<String> = input
        .iter()
        .map(|item| get_full_content(item))
        .collect::<Result<Vec<String>>>()?;

    utils::log(&format!("Reranking {} documents...", documents.len()));

    let document_refs: Vec<&String> = documents.iter().collect();
    let results = model
        .rerank(query, document_refs, true, None)
        .context("Failed to rerank documents")?;

    for result in results.iter() {
        let mut item = input[result.index].clone();
        item["rerank_score"] = json!(result.score);
        writeln!(stdout, "{}", serde_json::to_string(&item)?).context("Failed to write output")?;
    }

    utils::log("Reranking completed successfully.");
    Ok(())
}

fn get_full_content(item: &Value) -> Result<String> {
    let metadata = item["metadata"].as_object().context("Missing metadata")?;
    let file_path = metadata["file_path"]
        .as_str()
        .context("Missing file_path")?;
    let start_line = metadata["start_line"]
        .as_u64()
        .context("Missing start_line")? as usize;
    let end_line = metadata["end_line"].as_u64().context("Missing end_line")? as usize;

    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    Ok(content
        .lines()
        .skip(start_line)
        .take(end_line - start_line)
        .collect::<Vec<_>>()
        .join("\n"))
}

fn print_help() {
    eprintln!("vre - Vekta Reranker");
    eprintln!("Usage: vre <query> [-h|--help]");
    eprintln!();
    eprintln!("Reranks JSON-formatted documents based on the given query.");
    eprintln!("It's designed to work with Vekta text embedding results.");
    eprintln!("The tool reads JSON documents from stdin, one per line,");
    eprintln!("and outputs reranked JSON documents to stdout.");
    eprintln!();
    eprintln!("Each input JSON document should have a 'metadata' field with 'file_path',");
    eprintln!("'start_line', and 'end_line' subfields.");
    eprintln!("The output includes the original document fields plus a 'rerank_score' field.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help    Show this help message and exit");
    eprintln!();
    eprintln!("Example usage:");
    eprintln!("  cat top_k_results.jsonl | vre 'my search query' > reranked_results.jsonl");
}
