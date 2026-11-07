use mdictlib::{ChecksumPolicy, Limits, MddFile, MdxFile, OpenOptions};
use serde_json::{Value, json};
use std::collections::HashSet;

const MIB: usize = 1024 * 1024;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let path = arguments.first().ok_or("MDX path required")?;
    let media_argument = arguments.iter().position(|argument| argument == "--mdd");
    let query_end = media_argument.unwrap_or(arguments.len());
    if query_end == 0 {
        return Err("MDX path required before --mdd".into());
    }
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
    check_limit(
        "file_bytes",
        std::fs::metadata(path)?.len(),
        (128 * MIB) as u64,
    )?;
    let dictionary = MdxFile::open_with_options(path, &options)?;
    check_limit("entries", dictionary.len(), 1_000_000)?;
    let mut decoded_bytes = 0_u64;
    let mut decoded_key_bytes = 0_u64;
    let mut unique_keys = HashSet::new();
    for entry in dictionary.entries() {
        let entry = entry?;
        decoded_bytes += entry.text().len() as u64;
        decoded_key_bytes += entry.key().len() as u64;
        check_limit("total_body_bytes", decoded_bytes, (512 * MIB) as u64)?;
        check_limit("total_key_bytes", decoded_key_bytes, (64 * MIB) as u64)?;
        unique_keys.insert(entry.key().to_owned());
    }
    let mut queries = Vec::new();
    for query in &arguments[1..query_end] {
        let located = dictionary.locate(query)?;
        let mut matches = Vec::new();
        if let Some(located) = &located {
            for ordinal in located.iter() {
                let entry = dictionary
                    .entry_at(ordinal)?
                    .ok_or("matched entry missing")?;
                matches.push(json!({
                    "ordinal": ordinal.get(),
                    "key": entry.key(),
                    "body_bytes": entry.text().len(),
                }));
            }
        }
        queries.push(json!({
            "query": query,
            "basis": located.as_ref().map(|found| format!("{:?}", found.basis())),
            "matches": matches,
        }));
    }
    let media = if let Some(index) = media_argument {
        let path = arguments
            .get(index + 1)
            .ok_or("MDD path required after --mdd")?;
        Some(media_report(path, &arguments[index + 2..], &options)?)
    } else {
        None
    };
    let report: Value = json!({
        "entries": dictionary.len(),
        "unique_keys": unique_keys.len(),
        "duplicate_entries": dictionary.len() - unique_keys.len() as u64,
        "decoded_body_bytes": decoded_bytes,
        "engine_version": dictionary.header().generated_by_engine_version(),
        "encoding": dictionary.header().encoding_label(),
        "queries": queries,
        "mdd": media,
    });
    println!("{report}");
    Ok(())
}

fn media_report(path: &str, queries: &[String], options: &OpenOptions) -> Result<Value> {
    check_limit(
        "file_bytes",
        std::fs::metadata(path)?.len(),
        (128 * MIB) as u64,
    )?;
    let media = MddFile::open_with_options(path, options)?;
    check_limit("entries", media.len(), 1_000_000)?;
    let mut decoded_bytes = 0_u64;
    for resource in media.resources() {
        decoded_bytes += resource?.bytes().len() as u64;
        check_limit("total_media_bytes", decoded_bytes, (512 * MIB) as u64)?;
    }
    let mut results = Vec::new();
    for query in queries {
        let located = media.locate(query)?;
        let mut matches = Vec::new();
        if let Some(located) = located {
            for ordinal in located.iter() {
                let resource = media.resource_at(ordinal)?.ok_or("matched media missing")?;
                matches.push(json!({"key": resource.key(), "bytes": resource.bytes().len()}));
            }
        }
        results.push(json!({"query": query, "matches": matches}));
    }
    Ok(json!({
        "entries": media.len(),
        "decoded_bytes": decoded_bytes,
        "engine_version": media.header().generated_by_engine_version(),
        "queries": results,
    }))
}

fn check_limit(limit: &'static str, value: u64, max: u64) -> Result<()> {
    if value > max {
        return Err(mdictlib::Error::LimitExceeded { limit, value, max }.into());
    }
    Ok(())
}
