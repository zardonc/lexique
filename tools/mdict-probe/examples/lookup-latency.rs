use mdictlib::{ChecksumPolicy, Limits, MdxFile, OpenOptions};
use serde_json::json;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let path = arguments.first().ok_or("MDX path required")?;
    const MIB: usize = 1024 * 1024;
    assert!(std::fs::metadata(path)?.len() <= (128 * MIB) as u64);
    let options = OpenOptions::new()
        .with_checksum_policy(ChecksumPolicy::Verify)
        .with_limits(
            Limits::new()
                .with_header_xml_bytes(MIB)
                .with_compressed_block_bytes(16 * MIB)
                .with_decompressed_block_bytes(16 * MIB)
                .with_materialized_record_bytes(8 * MIB)
                .with_locator_entries(1_000_000)
                .with_working_memory_bytes(256 * MIB),
        );
    let opened = Instant::now();
    let dictionary = MdxFile::open_with_options(path, &options)?;
    let open_ms = opened.elapsed().as_secs_f64() * 1000.0;
    let before = dictionary.memory_usage()?;
    let mut queries = Vec::new();
    for query in &arguments[1..] {
        let mut timings = Vec::new();
        let mut records = 0;
        let mut body_bytes = 0;
        for _ in 0..3 {
            let started = Instant::now();
            records = 0;
            body_bytes = 0;
            if let Some(matches) = dictionary.locate(query)? {
                for ordinal in matches.iter() {
                    let entry = dictionary
                        .entry_at(ordinal)?
                        .ok_or("Missing matched ordinal")?;
                    records += 1;
                    body_bytes += entry.text().len();
                }
            }
            timings.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        queries.push(
            json!({"query":query,"records":records,"body_bytes":body_bytes,"milliseconds":timings}),
        );
    }
    let after = dictionary.memory_usage()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({"file_bytes":std::fs::metadata(path)?.len(),
        "open_ms":open_ms,"metadata_bytes":before.current_bytes(),"record_bytes_before_query":before.record_cache_bytes(),
        "current_bytes":after.current_bytes(),"peak_bytes":after.peak_bytes(),"queries":queries}))?
    );
    Ok(())
}
