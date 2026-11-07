use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use lexique_dictionary::conjugation::lookup;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.len() != 3 {
        return Err(
            "usage: compare_conjugation <resource.sqlite3> <sha256> <expected.jsonl>".into(),
        );
    }
    let mut count = 0;
    for line in BufReader::new(File::open(&arguments[2])?).lines() {
        let expected: serde_json::Value = serde_json::from_str(&line?)?;
        let key = expected["query"]
            .as_str()
            .ok_or("Expected a query string")?;
        let actual = serde_json::to_value(lookup(Path::new(&arguments[0]), key, &arguments[1])?)?;
        if actual != expected {
            return Err(format!("Native conjugation differs from baseline for {key}").into());
        }
        count += 1;
    }
    if count == 0 {
        return Err("Expected at least one baseline lookup".into());
    }
    println!("Verified {count} public conjugation lookups");
    Ok(())
}
