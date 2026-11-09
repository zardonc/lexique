use lexique_dictionary::mdx::Dictionary;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/mdict-probe/tests/fixtures")
        .join(name)
}

#[test]
fn mdx_lookup_keeps_duplicate_bodies_and_their_source() {
    let dictionary = Dictionary::open(&fixture("v2-utf8-none.mdx"), "mdx-1", "first.mdx").unwrap();
    let result = dictionary.lookup("double").unwrap();
    assert_eq!(result.dictionary_id.as_deref(), Some("mdx-1"));
    assert_eq!(result.source.title, "first.mdx");
    assert!(result.source.license.is_none());
    assert_eq!(
        result.sha256,
        "a09421f5f4b008eb9ab2ed9e4b988405b28ae0dbaea8232e95a7762e6b9c9aee"
    );
    assert_eq!(result.entries.len(), 2);
    assert_ne!(result.entries[0].ordinal, result.entries[1].ordinal);
    assert_eq!(
        result.entries[0].html.as_deref(),
        Some("<p>first sense</p>")
    );
    assert_eq!(
        result.entries[1].html.as_deref(),
        Some("<p>second sense</p>")
    );
    assert!(dictionary.lookup("missing").unwrap().entries.is_empty());
    assert_eq!(dictionary.lookup("\n").unwrap_err().code, "invalid_query");
}

#[test]
fn mdx_open_reports_unsupported_and_oversized_sources() {
    assert_eq!(
        Dictionary::open(&fixture("unsupported-v3-header.mdx"), "mdx-1", "v3.mdx")
            .err()
            .unwrap()
            .code,
        "unsupported"
    );
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/issue-9/parser-limits")
        .join(format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("oversized.mdx");
    std::fs::File::create(&path)
        .unwrap()
        .set_len(lexique_dictionary::mdx::MAX_FILE_BYTES as u64 + 1)
        .unwrap();
    assert_eq!(
        Dictionary::open(&path, "mdx-1", "oversized.mdx")
            .err()
            .unwrap()
            .code,
        "limit"
    );
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn mdx_response_limits_reject_excess_records_and_body_bytes() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/desktop/e2e/fixtures/mdx");
    for (file, key) in [
        ("response-count.mdx", "many"),
        ("response-bytes.mdx", "huge"),
    ] {
        let dictionary = Dictionary::open(&root.join(file), "mdx-1", file).unwrap();
        assert_eq!(dictionary.lookup(key).unwrap_err().code, "limit");
    }
}
