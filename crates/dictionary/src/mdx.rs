//! Bounded, duplicate-preserving MDX reads; HTML is returned for isolated display.

use mdictlib::{ChecksumPolicy, Limits, MdxFile, OpenOptions};
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path};

use crate::{DictionaryError, Entry, Lookup, Source, failure, validate_query};

const MIB: usize = 1024 * 1024;
pub const MAX_FILE_BYTES: usize = 128 * MIB;
pub const MAX_RESPONSE_BYTES: usize = 4 * MIB;

pub struct Dictionary {
    file: MdxFile,
    id: String,
    title: String,
    sha256: String,
}

fn io_error(error: std::io::Error) -> DictionaryError {
    failure(
        match error.kind() {
            std::io::ErrorKind::NotFound => "missing",
            std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::InvalidData => "corrupt",
            _ => "query_failed",
        },
        "Could not read MDX",
        error,
    )
}

fn parser_error(error: mdictlib::Error) -> DictionaryError {
    let code = match &error {
        mdictlib::Error::Unsupported(_) | mdictlib::Error::MissingPasscode => "unsupported",
        mdictlib::Error::LimitExceeded { .. } | mdictlib::Error::AllocationFailed { .. } => "limit",
        mdictlib::Error::Io(error) => match error.kind() {
            std::io::ErrorKind::NotFound => "missing",
            std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::InvalidData => "corrupt",
            _ => "query_failed",
        },
        _ => "corrupt",
    };
    failure(code, "MDX read failed", error)
}

fn hash(path: &Path) -> Result<String, DictionaryError> {
    let mut file = File::open(path).map_err(io_error)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(io_error)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

impl Dictionary {
    pub fn content_hash(&self) -> &str {
        &self.sha256
    }

    pub fn open(path: &Path, id: &str, title: &str) -> Result<Self, DictionaryError> {
        let length = std::fs::metadata(path).map_err(io_error)?.len();
        if length == 0 || length > MAX_FILE_BYTES as u64 {
            return Err(failure("limit", "MDX size", "non-empty, at most 128 MiB"));
        }
        let mut header = [0_u8; 4];
        File::open(path)
            .map_err(io_error)?
            .read_exact(&mut header)
            .map_err(io_error)?;
        let header_bytes = u32::from_be_bytes(header) as u64;
        if header_bytes == 0 || header_bytes + 8 > length {
            return Err(failure(
                "corrupt",
                "MDX header",
                "header extends beyond the file",
            ));
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
        let file = MdxFile::open_with_options(path, &options).map_err(parser_error)?;
        let encoding = file.header().encoding_label().unwrap_or("UTF-8");
        if !["UTF-8", "UTF-16", "UTF-16LE"]
            .iter()
            .any(|value| encoding.eq_ignore_ascii_case(value))
        {
            return Err(failure("unsupported", "MDX encoding", encoding));
        }
        if file.len() > 1_000_000 {
            return Err(failure("limit", "MDX entries", "maximum 1,000,000"));
        }
        let sha256 = hash(path)?;
        Ok(Self {
            file,
            id: id.into(),
            title: title.into(),
            sha256,
        })
    }

    pub fn lookup(&self, key: &str) -> Result<Lookup, DictionaryError> {
        validate_query(key)?;
        let mut entries = Vec::new();
        let mut bytes = 0;
        if let Some(matches) = self.file.locate(key).map_err(parser_error)? {
            if matches.len() > 200 {
                return Err(failure(
                    "limit",
                    "MDX response",
                    "maximum 200 matching records",
                ));
            }
            for ordinal in matches.iter() {
                let entry = self
                    .file
                    .entry_at(ordinal)
                    .map_err(parser_error)?
                    .ok_or_else(|| failure("corrupt", "MDX entry", "matched ordinal is absent"))?;
                bytes += entry.text().len() + entry.key().len();
                if bytes > MAX_RESPONSE_BYTES {
                    return Err(failure("limit", "MDX response", "maximum 4 MiB"));
                }
                entries.push(Entry {
                    ordinal: (ordinal.get() + 1) as u32,
                    key: entry.key().into(),
                    pronunciations: Vec::new(),
                    grammar: Vec::new(),
                    senses: Vec::new(),
                    html: Some(entry.text().into()),
                });
            }
        }
        Ok(Lookup {
            dictionary_id: Some(self.id.clone()),
            sha256: self.sha256.clone(),
            source: Source {
                title: self.title.clone(),
                version: self.sha256.clone(),
                attribution: None,
                license: None,
                source_url: None,
                license_url: None,
            },
            entries,
            warnings: Vec::new(),
        })
    }
}
