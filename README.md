# Vekta Embeddings

Vekta Embeddings is a set of Unix-style tools for text and image embedding, and reranking. These tools are designed to be used in bash scripts and command-line pipelines, making it easy for system administrators and non-AI professionals to leverage semantic search and analysis capabilities.

## Installation

1. Ensure you have Rust installed on your system. If not, install it from [https://rustup.rs/](https://rustup.rs/)

2. Clone this repository:
   ```
   git clone https://github.com/yourusername/vekta_embeddings.git
   cd vekta_embeddings
   ```

3. Build the tools:
   ```
   cargo build --release
   ```

4. The compiled binaries will be in the `target/release` directory. You can add this directory to your PATH or move the binaries to a directory in your PATH.

## Tools

### 1. vte (Vekta Text Embedder)

`vte` creates embeddings for text files.

Usage:

```bash
find /path/to/documents -name ".txt" | vte > text_embeddings.jsonl
```

This command will find all .txt files in the specified directory, create embeddings for each file, and save the results in JSONL format.

### 2. vie (Vekta Image Embedder)

`vie` creates embeddings for image files.

Usage:

```bash
find /path/to/images -name ".jpg" -o -name ".png" | vie > image_embeddings.jsonl
```

This command will find all .jpg and .png files in the specified directory, create embeddings for each image, and save the results in JSONL format.

### 3. vre (Vekta Reranker)

`vre` reranks a list of documents based on a query.

Usage:

```bash
cat top_k_results.jsonl | vre "your search query" > reranked_results.jsonl
```

This command takes a JSONL file of initial search results, reranks them based on the given query, and outputs the reranked results.

## Practical Examples

### Example 1: Semantic search in text documents

```bash

# Create embeddings for all text files in a directory
find /path/to/documents -name ".txt" | vte > document_embeddings.jsonl
```


## Notes

- The `vte`, `vie`, and `vre` tools are designed to work with Unix pipes and standard input/output.
- The tools automatically detect system resources and adjust batch sizes accordingly.
- Use the `-h` or `--help` option with each tool to see specific usage instructions.
- The output is in JSONL format, which can be easily processed with tools like `jq`.
- These tools are meant to be used in conjunction with a vector database like Vekta for efficient similarity search.

## Environment Variables

- `VEKTA_QUIET`: Set to "1" to suppress log messages from the tools.
