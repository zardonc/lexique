use crate::note_import::{NoteImportAction, NoteImportSelection};
use lexique_storage::{GrammarImportContent, GrammarImportResult, GrammarRecord, ImportSource};
use rusqlite::Connection;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct GrammarImportGroup {
    pub content: GrammarImportContent,
    pub row: u64,
    pub existing: Option<GrammarRecord>,
    pub status: &'static str,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GrammarImportPreview {
    pub operation: String,
    pub source: ImportSource,
    pub group: Option<GrammarImportGroup>,
    pub error: Option<String>,
}

pub fn preview_csv_bytes(
    connection: &Connection,
    source: ImportSource,
    bytes: &[u8],
    operation: &str,
) -> Result<GrammarImportPreview, String> {
    source.validate().map_err(|e| e.to_string())?;
    let mut preview = GrammarImportPreview {
        operation: operation.into(),
        source,
        group: None,
        error: None,
    };
    let parsed = (|| -> Result<(), String> {
        let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
        crate::note_import_csv::validate_quotes(bytes, b',')?;
        let mut reader = csv::Reader::from_reader(bytes);
        let header = reader
            .headers()
            .map_err(|e| format!("CSV header: {e}"))?
            .clone();
        let mut columns = std::collections::HashMap::new();
        for (index, name) in header.iter().enumerate() {
            if !["exchange_id", "title", "body", "question", "answer"].contains(&name) {
                return Err(format!(
                    "Expected canonical grammar CSV columns; unsupported column {name}"
                ));
            }
            if columns.insert(name, index).is_some() {
                return Err(format!("Duplicate canonical grammar column {name}"));
            }
        }
        if !columns.contains_key("title") {
            return Err("Grammar CSV requires the title column".into());
        }
        let mut rows = reader.records();
        let row = rows
            .next()
            .ok_or("Grammar CSV contains no record")?
            .map_err(|e| format!("CSV record: {e}"))?;
        if rows.next().is_some() {
            return Err("Import one grammar record at a time".into());
        }
        let field = |name| columns.get(name).map(|&index| row[index].to_string());
        let content = GrammarImportContent {
            exchange_id: field("exchange_id"),
            title: field("title").unwrap(),
            body: field("body"),
            question: field("question"),
            answer: field("answer"),
        };
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
        let error = content
            .validate()
            .err()
            .map(|e| format!("{} row {line}: {e}", preview.source.filename));
        let existing = if error.is_none() {
            content
                .exchange_id
                .as_deref()
                .filter(|id| !id.is_empty())
                .map(|id| lexique_storage::find_grammar_exchange(connection, id))
                .transpose()
                .map_err(|e| e.to_string())?
                .flatten()
        } else {
            None
        };
        let status = if error.is_some() {
            "invalid"
        } else if let Some(record) = &existing {
            if content.unchanged(record).map_err(|e| e.to_string())? {
                "identical"
            } else {
                "different"
            }
        } else {
            "create"
        };
        preview.group = Some(GrammarImportGroup {
            content,
            row: line,
            existing,
            status,
            error,
        });
        Ok(())
    })();
    if let Err(error) = parsed {
        preview.error = Some(format!("{}: {error}", preview.source.filename));
    }
    Ok(preview)
}

pub fn commit_preview(
    connection: &mut Connection,
    preview: &GrammarImportPreview,
    selections: &[NoteImportSelection],
) -> Result<GrammarImportResult, String> {
    if preview.error.is_some() {
        return Err("Resolve grammar CSV errors before importing".into());
    }
    let group = preview
        .group
        .as_ref()
        .ok_or("Grammar preview has no record")?;
    if group.error.is_some()
        || selections.len() != 1
        || selections[0].index != 0
        || !matches!(
            (group.status, &selections[0].action),
            ("create", NoteImportAction::Create) | ("identical", NoteImportAction::Skip)
        )
    {
        return Err(
            "Invalid grammar import decision; changed existing notes remain unchanged".into(),
        );
    }
    let now = crate::notes::now_millis(connection).map_err(|e| e.to_string())?;
    lexique_storage::save_grammar_import(
        connection,
        &preview.operation,
        &group.content,
        group.existing.as_ref(),
        now,
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "grammar_import_tests.rs"]
mod tests;
