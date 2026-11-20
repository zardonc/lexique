use crate::note_import::{NoteImportGroup, NoteImportMapping, NoteImportPreview};
use lexique_storage::{ImportSource, NoteImportContent};
use rusqlite::Connection;
use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

pub fn preview_zip_bytes(
    connection: &Connection,
    source: ImportSource,
    bytes: &[u8],
    operation: &str,
) -> Result<NoteImportPreview, String> {
    source.validate().map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("{}: invalid ZIP container: {error}", source.filename))?;
    archive
        .validate_header_consistency()
        .map_err(|error| format!("{}: invalid ZIP headers: {error}", source.filename))?;
    let mut preview = NoteImportPreview {
        operation: operation.into(),
        source,
        columns: vec![],
        mapping: NoteImportMapping::default(),
        groups: vec![],
        error: None,
    };
    let mut identities = HashMap::<String, usize>::new();
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
        let parsed = crate::note_markdown::preview_markdown_bytes(
            connection,
            preview.source.clone(),
            &bytes,
            operation,
        )?;
        if let Some(error) = parsed.error {
            preview.groups.push(NoteImportGroup {
                files: Some(vec![name.clone()]),
                rows: vec![index as u64 + 1],
                content: NoteImportContent {
                    headword: name.clone(),
                    notes: None,
                    examples: None,
                },
                existing: None,
                status: "invalid",
                error: Some(format!("{name}: {error}")),
                enrichment: None,
            });
            continue;
        }
        let mut group = parsed
            .groups
            .into_iter()
            .next()
            .ok_or("ZIP template has no note")?;
        group.content.headword = lexique_storage::normalize_headword(&group.content.headword);
        if let Some(&previous) = identities.get(&group.content.headword) {
            let existing = &mut preview.groups[previous];
            existing.rows.push(index as u64 + 1);
            existing.files.as_mut().unwrap().push(name);
            if existing.content != group.content {
                existing.status = "invalid";
                existing.error = Some("Same headword has conflicting provided fields or content; exclude the whole group".into());
            }
        } else {
            identities.insert(group.content.headword.clone(), preview.groups.len());
            group.files = Some(vec![name]);
            group.rows = vec![index as u64 + 1];
            preview.groups.push(group);
        }
    }
    Ok(preview)
}
