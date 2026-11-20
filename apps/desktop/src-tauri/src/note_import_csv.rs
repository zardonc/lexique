use crate::note_import::{
    NoteDelimiter, NoteImportGroup, NoteImportMapping, NoteImportOptions, NoteImportPreview,
};
use lexique_storage::{ImportSource, NoteExample, NoteFormat, NoteImportContent};
use rusqlite::Connection;
use std::{fs::File, io::Read, path::Path};

pub fn read_csv_file(path: &Path) -> Result<(ImportSource, Vec<u8>), String> {
    let source = ImportSource {
        format: NoteFormat::Csv,
        filename: path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("CSV filename is not UTF-8")?
            .into(),
    };
    source.validate().map_err(|e| e.to_string())?;
    let file = File::open(path).map_err(|e| format!("Could not open CSV: {e}"))?;
    let mut bytes = Vec::new();
    file.take(32 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Could not read CSV: {e}"))?;
    if bytes.len() > 32 * 1024 * 1024 {
        return Err("CSV exceeds 32 MiB".into());
    }
    Ok((source, bytes))
}
fn automatic_mapping(columns: &[String]) -> Result<NoteImportMapping, String> {
    let mut mapping = NoteImportMapping::default();
    for (name, target) in [
        ("headword", &mut mapping.headword),
        ("notes", &mut mapping.notes),
        ("example_fr", &mut mapping.example_fr),
        ("example_zh", &mut mapping.example_zh),
    ] {
        for (index, column) in columns
            .iter()
            .enumerate()
            .filter(|(_, column)| column.as_str() == name)
        {
            if target.replace(index).is_some() {
                return Err(format!(
                    "Duplicate canonical column {column}; choose a mapping"
                ));
            }
        }
    }
    Ok(mapping)
}
fn validate_mapping(mapping: &NoteImportMapping, columns: usize) -> Result<(), String> {
    if mapping.headword.is_none() {
        return Err("Map the required headword column".into());
    }
    let mut used = std::collections::HashSet::new();
    for index in [
        mapping.headword,
        mapping.notes,
        mapping.example_fr,
        mapping.example_zh,
    ]
    .into_iter()
    .flatten()
    {
        if index >= columns {
            return Err("Mapped column is outside the CSV header".into());
        }
        if !used.insert(index) {
            return Err("Each column may map to only one field".into());
        }
    }
    Ok(())
}

fn validate_quotes(bytes: &[u8], delimiter: u8) -> Result<(), String> {
    // rust-csv accepts incomplete quoting; reject it before interpreting user fields.
    let mut state = 0;
    let mut line = 1;
    for &byte in bytes {
        state = match (state, byte) {
            (0, b'"') | (3, b'"') => 2,
            (2, b'"') => 3,
            (2, _) => 2,
            (_, byte) if byte == delimiter || byte == b'\r' || byte == b'\n' => 0,
            (3, _) | (1, b'"') => return Err(format!("CSV row {line}: malformed quoted field")),
            _ => 1,
        };
        if byte == b'\n' {
            line += 1;
        }
    }
    if state == 2 {
        return Err(format!("CSV row {line}: unclosed quoted field"));
    }
    Ok(())
}
pub fn preview_csv_bytes(
    connection: &Connection,
    source: ImportSource,
    bytes: &[u8],
    operation: &str,
    options: &NoteImportOptions,
) -> Result<NoteImportPreview, String> {
    source.validate().map_err(|e| e.to_string())?;
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let delimiter = match options.delimiter {
        NoteDelimiter::Comma => b',',
        NoteDelimiter::Semicolon => b';',
    };
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_reader(bytes);
    let mut preview = NoteImportPreview {
        operation: operation.into(),
        source,
        columns: vec![],
        mapping: options.mapping.clone().unwrap_or_default(),
        groups: vec![],
        error: None,
    };
    let parsed = (|| -> Result<(), String> {
        validate_quotes(bytes, delimiter)?;
        preview.columns = reader
            .headers()
            .map_err(|e| format!("CSV header: {e}"))?
            .iter()
            .map(String::from)
            .collect();
        if options.mapping.is_none() {
            preview.mapping = automatic_mapping(&preview.columns)?;
        }
        validate_mapping(&preview.mapping, preview.columns.len())?;
        let mapping = &preview.mapping;
        let mut identities = std::collections::HashMap::<String, usize>::new();
        for row in reader.records() {
            let row = row.map_err(|e| format!("CSV record: {e}"))?;
            let line = row.position().map(|p| p.line()).unwrap_or(0);
            let headword = lexique_storage::normalize_headword(&row[mapping.headword.unwrap()]);
            let notes = mapping.notes.map(|index| row[index].to_string());
            let example = NoteExample {
                fr: mapping
                    .example_fr
                    .map(|index| row[index].into())
                    .unwrap_or_default(),
                zh: mapping
                    .example_zh
                    .map(|index| row[index].into())
                    .unwrap_or_default(),
            };
            let supplied_examples = mapping.example_fr.is_some() || mapping.example_zh.is_some();
            let index = *identities.entry(headword.clone()).or_insert_with(|| {
                let index = preview.groups.len();
                preview.groups.push(NoteImportGroup {
                    content: NoteImportContent {
                        headword,
                        notes: notes.clone(),
                        examples: supplied_examples.then(Vec::new),
                    },
                    rows: vec![],
                    existing: None,
                    status: "invalid",
                    error: None,
                });
                index
            });
            let group = &mut preview.groups[index];
            group.rows.push(line);
            if group.content.notes != notes {
                group.error = Some(format!(
                    "Row {line}, {}: conflicting notes in the same headword group",
                    group.content.headword
                ));
            }
            if let Some(examples) = &mut group.content.examples {
                if !example.fr.is_empty() || !example.zh.is_empty() {
                    examples.push(example);
                }
            }
            if let Err(error) = group.content.create_request(operation) {
                group.error = Some(format!("Row {line}, {}: {error}", group.content.headword));
            }
        }
        if preview.groups.is_empty() {
            return Err("CSV has no vocabulary records".into());
        }
        for group in &mut preview.groups {
            if group.error.is_some() {
                continue;
            }
            group.existing = lexique_storage::find(connection, &group.content.headword)
                .map_err(|e| e.to_string())?;
            group.status = match &group.existing {
                Some(record) if group.content.matches(record) => "identical",
                Some(_) => "different",
                None => "create",
            };
        }
        Ok(())
    })();
    if let Err(error) = parsed {
        preview.groups.clear();
        preview.error = Some(error);
    }
    Ok(preview)
}
