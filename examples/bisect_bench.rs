use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, TEXT};
use tantivy::{doc, Index, IndexWriter};
use std::time::Instant;

fn main() -> tantivy::Result<()> {
    let mut schema_builder = Schema::builder();
    let text_field = schema_builder.add_text_field("text", TEXT);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema.clone());
    let mut index_writer: IndexWriter = index.writer(50_000_000)?;

    // Create realistic dataset
    for i in 0..100_000 {
        index_writer.add_document(doc!(text_field => format!("griffith observatory document {} with additional text content", i)))?;
    }
    index_writer.commit()?;

    let reader = index.reader()?;
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![text_field]);

    // Union query - matches what the benchmark tests
    let query = query_parser.parse_query("griffith observatory")?;

    // Warmup
    for _ in 0..500 {
        let _ = searcher.search(&query, &TopDocs::with_limit(100).order_by_score())?;
    }

    // Benchmark TOP_100 (this is what the search-benchmark-game uses)
    let iterations = 20_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = searcher.search(&query, &TopDocs::with_limit(100).order_by_score())?;
    }
    let elapsed = start.elapsed();

    let micros = elapsed.as_micros() / iterations;
    println!("AVG_MICROS: {}", micros);

    Ok(())
}
