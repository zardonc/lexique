use crate::note_import::{NoteDelimiter, NoteImportAction, NoteImportOptions, NoteImportSelection};
use lexique_storage::{
    GrammarImportContent, GrammarImportItem, GrammarImportResult, GrammarRecord, ImportSource,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GrammarImportMapping {
    pub exchange_id: Option<usize>,
    pub title: Option<usize>,
    pub body: Option<usize>,
    pub question: Option<usize>,
    pub answer: Option<usize>,
}
impl GrammarImportMapping {
    fn automatic(columns: &[String]) -> Result<Self, String> {
        let mut mapping = Self::default();
        for (name, target) in [
            ("exchange_id", &mut mapping.exchange_id),
            ("title", &mut mapping.title),
            ("body", &mut mapping.body),
            ("question", &mut mapping.question),
            ("answer", &mut mapping.answer),
        ] {
            for (index, _) in columns
                .iter()
                .enumerate()
                .filter(|(_, column)| column.as_str() == name)
            {
                if target.replace(index).is_some() {
                    return Err(format!(
                        "Duplicate canonical column {name}; choose a mapping"
                    ));
                }
            }
        }
        Ok(mapping)
    }
    fn validate(&self, columns: usize) -> Result<(), String> {
        if self.title.is_none() {
            return Err("Map the required title column".into());
        }
        let mut used = std::collections::HashSet::new();
        for index in [
            self.exchange_id,
            self.title,
            self.body,
            self.question,
            self.answer,
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
}
#[derive(Clone, Debug, Serialize)]
pub struct GrammarImportGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,
    pub content: GrammarImportContent,
    pub rows: Vec<u64>,
    pub existing: Option<GrammarRecord>,
    pub status: &'static str,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct GrammarImportPreview {
    pub operation: String,
    pub source: ImportSource,
    pub columns: Vec<String>,
    pub mapping: GrammarImportMapping,
    pub groups: Vec<GrammarImportGroup>,
    pub error: Option<String>,
}

fn match_grammar_content(
    connection: &Connection,
    content: &GrammarImportContent,
) -> Result<(Option<GrammarRecord>, &'static str), String> {
    let existing = content
        .exchange_id
        .as_deref()
        .filter(|id| !id.is_empty())
        .map(|id| lexique_storage::find_grammar_exchange(connection, id))
        .transpose()
        .map_err(|error| error.to_string())?
        .flatten();
    let status = match &existing {
        Some(record)
            if content
                .unchanged(record)
                .map_err(|error| error.to_string())? =>
        {
            "identical"
        }
        Some(_) => "different",
        None => "create",
    };
    Ok((existing, status))
}

pub fn preview_markdown_bytes(
    connection: &Connection,
    source: ImportSource,
    bytes: &[u8],
    operation: &str,
) -> Result<GrammarImportPreview, String> {
    source.validate().map_err(|error| error.to_string())?;
    let mut preview = GrammarImportPreview {
        operation: operation.into(),
        source,
        columns: vec![],
        mapping: GrammarImportMapping::default(),
        groups: vec![],
        error: None,
    };
    let content = std::str::from_utf8(bytes)
        .map_err(|error| format!("Markdown must be UTF-8: {error}"))
        .and_then(crate::note_markdown::parse_grammar_markdown);
    match content {
        Ok(content) => {
            let (existing, status) = match_grammar_content(connection, &content)?;
            preview.groups.push(GrammarImportGroup {
                files: None,
                content,
                rows: vec![1],
                existing,
                status,
                error: None,
            });
        }
        Err(error) => preview.error = Some(format!("{}: {error}", preview.source.filename)),
    }
    Ok(preview)
}

pub fn preview_zip_bytes(
    connection: &Connection,
    source: ImportSource,
    bytes: &[u8],
    operation: &str,
) -> Result<GrammarImportPreview, String> {
    use std::io::{Cursor, Read};
    source.validate().map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("{}: unreadable ZIP: {error}", source.filename))?;
    let mut preview = GrammarImportPreview {
        operation: operation.into(),
        source,
        columns: vec![],
        mapping: GrammarImportMapping::default(),
        groups: vec![],
        error: None,
    };
    let mut identities = std::collections::HashMap::<String, usize>::new();
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index_with_options(
                index,
                zip::ZipReadOptions::new().enforce_header_consistency(true),
            )
            .map_err(|error| {
                format!(
                    "{}: unreadable ZIP entry {}: {error}",
                    preview.source.filename,
                    index + 1
                )
            })?;
        let name = entry
            .name()
            .map_err(|error| {
                format!(
                    "{}: invalid ZIP entry name: {error}",
                    preview.source.filename
                )
            })?
            .into_owned();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(|error| {
            format!(
                "{}: unreadable ZIP entry {name}: {error}",
                preview.source.filename
            )
        })?;
        let parsed = std::str::from_utf8(&bytes)
            .map_err(|error| format!("Markdown must be UTF-8: {error}"))
            .and_then(|text| crate::note_markdown::parse_grammar_markdown_fields(text, true));
        let content = match parsed {
            Ok(content) => content,
            Err(error) => {
                preview.groups.push(GrammarImportGroup {
                    files: Some(vec![name.clone()]),
                    rows: vec![index as u64 + 1],
                    content: GrammarImportContent {
                        exchange_id: None,
                        title: name.clone(),
                        body: None,
                        question: None,
                        answer: None,
                    },
                    existing: None,
                    status: "invalid",
                    error: Some(format!("{}: {name}: {error}", preview.source.filename)),
                });
                continue;
            }
        };
        if let Some(previous) = content
            .exchange_id
            .as_ref()
            .filter(|id| !id.is_empty())
            .and_then(|id| identities.get(id).copied())
        {
            let existing = &mut preview.groups[previous];
            existing.rows.push(index as u64 + 1);
            existing
                .files
                .as_mut()
                .expect("ZIP group has files")
                .push(name);
            if existing.content != content {
                existing.status = "invalid";
                existing.error = Some("conflicting inputs for the same exchange identity".into());
            }
        } else {
            if let Some(id) = content.exchange_id.as_ref().filter(|id| !id.is_empty()) {
                identities.insert(id.clone(), preview.groups.len());
            }
            preview.groups.push(GrammarImportGroup {
                files: Some(vec![name]),
                rows: vec![index as u64 + 1],
                content,
                existing: None,
                status: "invalid",
                error: None,
            });
        }
    }
    for group in &mut preview.groups {
        if group.error.is_none() {
            group.error = group
                .content
                .validate()
                .err()
                .map(|error| error.to_string());
        }
        if let Some(error) = &group.error {
            group.error = Some(format!(
                "{}: {}: {error}",
                preview.source.filename,
                group.files.as_ref().unwrap().join(", ")
            ));
            continue;
        }
        (group.existing, group.status) = match_grammar_content(connection, &group.content)?;
    }
    if preview.groups.is_empty() {
        preview.error = Some(format!(
            "{}: ZIP contains no grammar records",
            preview.source.filename
        ));
    }
    Ok(preview)
}
#[cfg(test)]
pub fn preview_csv_bytes(
    connection: &Connection,
    source: ImportSource,
    bytes: &[u8],
    operation: &str,
) -> Result<GrammarImportPreview, String> {
    preview_csv_with_options(
        connection,
        source,
        bytes,
        operation,
        &NoteImportOptions::default(),
    )
}
pub fn preview_csv_with_options(
    connection: &Connection,
    source: ImportSource,
    bytes: &[u8],
    operation: &str,
    options: &NoteImportOptions,
) -> Result<GrammarImportPreview, String> {
    source.validate().map_err(|e| e.to_string())?;
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let delimiter = match options.delimiter {
        NoteDelimiter::Comma => b',',
        NoteDelimiter::Semicolon => b';',
    };
    let mut preview = GrammarImportPreview {
        operation: operation.into(),
        source,
        columns: vec![],
        mapping: options.grammar_mapping.clone().unwrap_or_default(),
        groups: vec![],
        error: None,
    };
    let parsed = (|| -> Result<(), String> {
        crate::note_import_csv::validate_quotes(bytes, delimiter)?;
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .flexible(true)
            .from_reader(bytes);
        preview.columns = reader
            .headers()
            .map_err(|e| format!("CSV header: {e}"))?
            .iter()
            .map(String::from)
            .collect();
        if options.grammar_mapping.is_none() {
            preview.mapping = GrammarImportMapping::automatic(&preview.columns)?;
        }
        preview.mapping.validate(preview.columns.len())?;
        let mut identities = std::collections::HashMap::<String, usize>::new();
        for row in reader.records() {
            let row = row.map_err(|e| format!("CSV record: {e}"))?;
            let offset = row.position().map(|p| p.byte() as usize).unwrap_or(0);
            let separators = bytes[offset..]
                .iter()
                .take_while(|byte| matches!(byte, b'\r' | b'\n'))
                .count();
            let prefix = &bytes[..offset + separators];
            let line = 1 + prefix
                .iter()
                .enumerate()
                .filter(|(index, byte)| {
                    **byte == b'\n' || **byte == b'\r' && prefix.get(index + 1) != Some(&b'\n')
                })
                .count() as u64;
            let field =
                |index: Option<usize>| index.and_then(|index| row.get(index)).map(String::from);
            let content = GrammarImportContent {
                exchange_id: field(preview.mapping.exchange_id),
                title: field(preview.mapping.title).unwrap_or_default(),
                body: field(preview.mapping.body),
                question: field(preview.mapping.question),
                answer: field(preview.mapping.answer),
            };
            let index = content
                .exchange_id
                .as_ref()
                .filter(|id| !id.is_empty())
                .and_then(|id| identities.get(id).copied());
            if let Some(index) = index {
                let group = &mut preview.groups[index];
                group.rows.push(line);
                if group.content != content {
                    group.error = Some("conflicting inputs for the same exchange identity".into());
                }
                if row.len() != preview.columns.len() {
                    group.error = Some("CSV record length differs from header".into());
                }
            } else {
                let index = preview.groups.len();
                if let Some(id) = content.exchange_id.as_ref().filter(|id| !id.is_empty()) {
                    identities.insert(id.clone(), index);
                }
                preview.groups.push(GrammarImportGroup {
                    files: None,
                    content,
                    rows: vec![line],
                    existing: None,
                    status: "invalid",
                    error: (row.len() != preview.columns.len())
                        .then(|| "CSV record length differs from header".into()),
                });
            }
        }
        if preview.groups.is_empty() {
            return Err("Grammar CSV contains no record".into());
        }
        for group in &mut preview.groups {
            if group.error.is_none() {
                group.error = group.content.validate().err().map(|e| e.to_string());
            }
            if let Some(reason) = &group.error {
                group.error = Some(format!(
                    "{} row {}: {reason}",
                    preview.source.filename,
                    group
                        .rows
                        .iter()
                        .map(u64::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                continue;
            }
            (group.existing, group.status) = match_grammar_content(connection, &group.content)?;
        }
        Ok(())
    })();
    if let Err(error) = parsed {
        preview.groups.clear();
        preview.error = Some(format!("{}: {error}", preview.source.filename));
    }
    Ok(preview)
}
pub fn commit_preview(
    connection: &mut Connection,
    preview: &GrammarImportPreview,
    selections: &[NoteImportSelection],
) -> Result<GrammarImportResult, String> {
    if preview.error.is_some() || selections.len() != preview.groups.len() {
        return Err("Resolve file structure and mapping, then decide every grammar group".into());
    }
    let mut decisions = vec![None; preview.groups.len()];
    for selection in selections {
        let decision = decisions
            .get_mut(selection.index)
            .ok_or("Invalid grammar import group index")?;
        if decision.replace(&selection.action).is_some() {
            return Err("Duplicate grammar import decision".into());
        }
    }
    let mut items = vec![];
    let mut skipped = 0;
    for (source_index, (group, decision)) in preview.groups.iter().zip(decisions).enumerate() {
        let decision = decision.ok_or("Decide every grammar import group")?;
        if *decision == NoteImportAction::Skip {
            skipped += 1;
            continue;
        }
        if group.error.is_some()
            || !matches!(
                (group.status, decision),
                ("create", NoteImportAction::Create) | ("different", NoteImportAction::Update)
            )
        {
            return Err("Invalid grammar import decision; choose create, update or skip".into());
        }
        items.push(GrammarImportItem {
            source_index,
            content: group.content.clone(),
            existing: group.existing.clone(),
        });
    }
    let now = crate::notes::now_millis(connection).map_err(|e| e.to_string())?;
    lexique_storage::save_grammar_import_batch(connection, &preview.operation, &items, skipped, now)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "grammar_import_tests.rs"]
mod tests;
