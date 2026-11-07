use mdictlib::{MddFile, MdxFile};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn probe(arguments: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_lexique-mdx-resource-probe"))
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn reject_file(name: &str, data: &[u8], reason: &str) {
    use std::io::Write;
    let path = std::env::temp_dir().join(format!("lexique-mdict-{}-{name}", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    file.write_all(data).unwrap();
    drop(file);
    let mut command = Command::new(env!("CARGO_BIN_EXE_lexique-mdx-resource-probe"));
    if name.ends_with(".mdd") {
        command
            .arg(fixture("v2-utf8-zlib.mdx"))
            .arg("être")
            .arg("--mdd");
    }
    let output = command.arg(&path).output().unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(!output.status.success(), "invalid source accepted");
    assert!(output.stdout.is_empty(), "partial success report emitted");
    let error = String::from_utf8_lossy(&output.stderr).to_lowercase();
    assert!(error.contains(reason), "{error}");
}

#[test]
fn rejects_a_bad_header_checksum() {
    for extension in ["mdx", "mdd"] {
        let mut data = std::fs::read(fixture(&format!("v2-utf8-zlib.{extension}"))).unwrap();
        let header_bytes = u32::from_be_bytes(data[..4].try_into().unwrap()) as usize;
        data[4 + header_bytes] ^= 1;
        reject_file(&format!("bad-checksum.{extension}"), &data, "checksum");
    }
}

fn v2_record_header(data: &[u8]) -> usize {
    let key_header = 8 + u32::from_be_bytes(data[..4].try_into().unwrap()) as usize;
    let key_index =
        u64::from_be_bytes(data[key_header + 24..key_header + 32].try_into().unwrap()) as usize;
    let key_blocks =
        u64::from_be_bytes(data[key_header + 32..key_header + 40].try_into().unwrap()) as usize;
    key_header + 44 + key_index + key_blocks
}

#[test]
fn rejects_a_bad_record_checksum() {
    for extension in ["mdx", "mdd"] {
        let mut data = std::fs::read(fixture(&format!("v2-utf8-none.{extension}"))).unwrap();
        let header = v2_record_header(&data);
        let index_size =
            u64::from_be_bytes(data[header + 16..header + 24].try_into().unwrap()) as usize;
        data[header + 32 + index_size + 4] ^= 1;
        reject_file(
            &format!("bad-record-checksum.{extension}"),
            &data,
            "checksum",
        );
    }
}

#[test]
fn rejects_truncated_headers_and_record_payloads() {
    for extension in ["mdx", "mdd"] {
        let data = std::fs::read(fixture(&format!("v2-utf8-zlib.{extension}"))).unwrap();
        reject_file(
            &format!("truncated-header.{extension}"),
            &data[..3],
            "error",
        );
        reject_file(
            &format!("truncated-record.{extension}"),
            &data[..data.len() - 1],
            "error",
        );
    }
}

#[test]
fn rejects_an_unsupported_version_header() {
    let data = std::fs::read(fixture("unsupported-v3-header.mdx")).unwrap();
    reject_file("unsupported-v3.mdx", &data, "unsupported");
}

#[test]
fn rejects_a_record_block_over_the_decompression_limit() {
    let mut data = std::fs::read(fixture("v2-utf8-zlib.mdx")).unwrap();
    let record_header = v2_record_header(&data);
    data[record_header + 40..record_header + 48]
        .copy_from_slice(&(16_u64 * 1024 * 1024 + 1).to_be_bytes());
    reject_file(
        "oversized-block.mdx",
        &data,
        "record_block_decompressed_bytes",
    );
}

#[test]
fn rejects_a_file_over_the_size_limit() {
    let path = std::env::temp_dir().join(format!(
        "lexique-mdict-{}-oversized.mdx",
        std::process::id()
    ));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    file.set_len(128 * 1024 * 1024 + 1).unwrap();
    drop(file);
    let output = Command::new(env!("CARGO_BIN_EXE_lexique-mdx-resource-probe"))
        .arg(&path)
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("file_bytes"));
}

#[test]
fn paired_formats_preserve_french_spellings_and_missing_results() {
    for name in [
        "v1-utf8-none",
        "v1-utf16-none",
        "v2-utf8-none",
        "v2-utf8-zlib",
        "v2-utf16-zlib",
        "v2-utf8-encrypted-index",
    ] {
        let source = fixture(&format!("{name}.mdx"));
        let media = fixture(&format!("{name}.mdd"));
        let words = [
            "être",
            "œil",
            "aujourd'hui",
            "aujourd’hui",
            "cote",
            "côte",
            "côté",
        ];
        let mut arguments = vec![source.to_str().unwrap()];
        arguments.extend(words);
        arguments.extend([
            "missing",
            "--mdd",
            media.to_str().unwrap(),
            "\\img\\œil.png",
        ]);
        let report = probe(&arguments);
        assert_eq!(report["entries"], 10, "{name}");
        assert_eq!(report["duplicate_entries"], 1, "{name}");
        for (index, word) in words.iter().enumerate() {
            assert_eq!(
                report["queries"][index]["basis"], "RawExact",
                "{name}: {word}"
            );
            assert_eq!(
                report["queries"][index]["matches"][0]["key"], *word,
                "{name}"
            );
        }
        assert_eq!(report["queries"][7]["matches"], json!([]), "{name}");
        assert_eq!(report["mdd"]["entries"], 3, "{name}");
        assert_eq!(
            report["mdd"]["queries"][0]["matches"][0]["bytes"], 70,
            "{name}"
        );
    }
}

#[test]
fn queries_preserve_both_duplicate_records() {
    let source = fixture("v2-utf8-zlib.mdx");
    let report = probe(&[source.to_str().unwrap(), "double"]);
    assert_eq!(report["entries"], 10);
    assert_eq!(report["duplicate_entries"], 1);
    assert_eq!(report["queries"][0]["matches"].as_array().unwrap().len(), 2);
    assert_eq!(report["queries"][0]["matches"][0]["body_bytes"], json!(18));
    assert_eq!(report["queries"][0]["matches"][1]["body_bytes"], json!(19));
}

#[test]
fn paired_entry_references_resolve_to_original_media() {
    let source = fixture("v2-utf8-zlib.mdx");
    let media = fixture("v2-utf8-zlib.mdd");
    let report = probe(&[
        source.to_str().unwrap(),
        "être",
        "--mdd",
        media.to_str().unwrap(),
        "\\theme.css",
        "\\img\\œil.png",
        "\\audio\\bonjour.wav",
        "\\missing.png",
    ]);
    assert_eq!(report["mdd"]["entries"], 3);
    assert_eq!(report["mdd"]["queries"][3]["matches"], json!([]));
    let dictionary = MdxFile::open(source).unwrap();
    assert_eq!(
        dictionary.lookup("être").unwrap().unwrap().text(),
        "<p>state</p><link href=\"theme.css\"><img src=\"img/œil.png\"><a href=\"sound://audio/bonjour.wav\">sound</a>"
    );
    let resources = MddFile::open(media).unwrap();
    for (index, (key, original)) in [
        ("\\theme.css", "theme.css"),
        ("\\img\\œil.png", "œil.png"),
        ("\\audio\\bonjour.wav", "bonjour.wav"),
    ]
    .iter()
    .enumerate()
    {
        let expected = std::fs::read(fixture(original)).unwrap();
        assert_eq!(resources.lookup(key).unwrap().unwrap().bytes(), expected);
        assert_eq!(
            report["mdd"]["queries"][index]["matches"][0]["bytes"],
            expected.len()
        );
    }
}
