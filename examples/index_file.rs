//! Index a JSONL file in cluster order using a serialized Tantivy schema.
//!
//! cargo run --release --example index_file -- schema.json documents.jsonl status service
//! Grouping fields are listed from outermost to innermost and may use JSON paths.

use std::fs::File;
use std::io::{BufRead, BufReader};

use tantivy::directory::RamDirectory;
use tantivy::schema::Schema;
use tantivy::{ClusteringConfig, Index, SingleSegmentIndexWriter, TantivyDocument};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        return Err("usage: index_file SCHEMA.json DOCUMENTS.jsonl GROUP_FIELD...".into());
    }
    let schema: Schema = serde_json::from_reader(BufReader::new(File::open(&args[0])?))?;
    let input = BufReader::new(File::open(&args[1])?);
    let mut writer: SingleSegmentIndexWriter = Index::builder()
        .schema(schema.clone())
        .single_segment_index_writer(RamDirectory::default(), 64_000_000)?
        .with_clustering(ClusteringConfig {
            fields: args[2..].to_vec(),
        })?;

    // Documents are staged until finalization. This writer does not automatically flush.
    for line in input.lines() {
        writer.add_document(TantivyDocument::parse_json(&schema, &line?)?)?;
    }
    let index = writer.finalize()?;
    let reader = index.reader()?;
    println!("Indexed {} documents", reader.searcher().num_docs());
    Ok(())
}
