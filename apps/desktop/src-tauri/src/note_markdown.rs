use lexique_storage::{NoteExample, NoteImportContent};

#[cfg(test)]
pub fn preview_markdown_file(
    connection: &rusqlite::Connection,
    path: &std::path::Path,
    operation: &str,
) -> Result<crate::note_import::NoteImportPreview, String> {
    let (source, bytes) =
        crate::note_import::read_note_file(path, lexique_storage::NoteFormat::Markdown)?;
    preview_markdown_bytes(connection, source, &bytes, operation)
}

pub fn preview_markdown_bytes(
    connection: &rusqlite::Connection,
    source: lexique_storage::ImportSource,
    bytes: &[u8],
    operation: &str,
) -> Result<crate::note_import::NoteImportPreview, String> {
    use crate::note_import::{NoteImportGroup, NoteImportMapping, NoteImportPreview};
    source.validate().map_err(|error| error.to_string())?;
    let mut preview = NoteImportPreview {
        operation: operation.into(),
        source,
        columns: vec![],
        mapping: NoteImportMapping::default(),
        groups: vec![],
        error: None,
    };
    let parsed = std::str::from_utf8(bytes)
        .map_err(|error| format!("Markdown must be UTF-8: {error}"))
        .and_then(parse_note_markdown);
    match parsed {
        Ok(content) => {
            let existing = lexique_storage::find(connection, &content.headword)
                .map_err(|error| error.to_string())?;
            let status = match &existing {
                Some(record) if content.matches(record) => "identical",
                Some(_) => "different",
                None => "create",
            };
            preview.groups.push(NoteImportGroup {
                content,
                existing,
                status,
                rows: vec![1],
                error: None,
                enrichment: None,
            });
        }
        Err(error) => preview.error = Some(format!("{}: {error}", preview.source.filename)),
    }
    Ok(preview)
}

pub fn note_markdown_filename(record: &lexique_storage::NoteExportRecord) -> String {
    let readable: String = record
        .headword
        .chars()
        .take(80)
        .map(|ch| {
            if ch.is_control() || "<>:\"/\\|?*".contains(ch) {
                '_'
            } else {
                ch
            }
        })
        .collect();
    let readable = readable.trim_matches([' ', '.']);
    format!(
        "{}-{}.md",
        if readable.is_empty() {
            "note"
        } else {
            readable
        },
        record.id
    )
}

pub fn format_note_markdown(record: &lexique_storage::NoteExportRecord) -> String {
    fn field(output: &mut String, heading: &str, text: &str) {
        let length = text.split(|ch| ch != '`').map(str::len).max().unwrap_or(0);
        let fence = "`".repeat((length + 1).max(3));
        output.push_str(&format!("{heading}\n\n{fence}text\n{text}\n{fence}\n\n"));
    }
    let mut output = "# Lexique note v1\n\n".to_string();
    field(&mut output, "## headword", &record.headword);
    field(&mut output, "## notes", &record.notes);
    output.push_str("## examples\n\n");
    for (index, example) in record.examples.iter().enumerate() {
        output.push_str(&format!("### {}\n\n", index + 1));
        field(&mut output, "#### fr", &example.fr);
        field(&mut output, "#### zh", &example.zh);
    }
    output
}

struct Template<'a> {
    rest: &'a str,
}

impl<'a> Template<'a> {
    fn line(&mut self) -> Option<(&'a str, &'static str)> {
        if self.rest.is_empty() {
            return None;
        }
        let (line, rest) = self.rest.split_once('\n').unwrap_or((self.rest, ""));
        self.rest = rest;
        Some(match line.strip_suffix('\r') {
            Some(line) => (line, "\r\n"),
            None => (line, "\n"),
        })
    }

    fn peek(&mut self) -> Option<&'a str> {
        while !self.rest.is_empty() {
            let line = self.rest.split('\n').next().unwrap();
            let line = line.strip_suffix('\r').unwrap_or(line);
            if !line.is_empty() {
                return Some(line);
            }
            self.line();
        }
        None
    }

    fn heading(&mut self) -> Option<&'a str> {
        self.peek()?;
        self.line().map(|(line, _)| line)
    }

    fn text(&mut self) -> Result<String, String> {
        self.peek();
        let (opening, separator) = self.line().ok_or("Missing text fence")?;
        let length = opening.bytes().take_while(|byte| *byte == b'`').count();
        if length < 3 || &opening[length..] != "text" {
            return Err("Expected a text fence (at least three backticks followed by text)".into());
        }
        let fence = &opening[..length];
        let payload = self.rest;
        while !self.rest.is_empty() {
            let before = self.rest;
            let (line, _) = self.line().unwrap();
            if line == fence {
                let value = &payload[..payload.len() - before.len()];
                // The opening fence determines the structural separator; payload CR stays intact.
                return Ok(value.strip_suffix(separator).unwrap_or(value).into());
            }
        }
        Err("Unclosed text fence".into())
    }

    fn examples(&mut self) -> Result<Vec<NoteExample>, String> {
        let mut examples = Vec::new();
        let mut number = 1;
        while self.peek().is_some_and(|line| !line.starts_with("## ")) {
            if self.heading() != Some(format!("### {number}").as_str()) {
                return Err(format!(
                    "Expected consecutive unique example number ### {number}"
                ));
            }
            let (mut fr, mut zh) = (None, None);
            while self.peek().is_some_and(|line| line.starts_with("#### ")) {
                let field = match self.heading().unwrap() {
                    "#### fr" => &mut fr,
                    "#### zh" => &mut zh,
                    other => return Err(format!("Unknown example language field: {other}")),
                };
                if field.is_some() {
                    return Err("Duplicate example language field".into());
                }
                *field = Some(self.text()?);
            }
            if fr.is_none() && zh.is_none() {
                return Err("Example requires fr or zh with a text fence".into());
            }
            examples.push(NoteExample {
                fr: fr.unwrap_or_default(),
                zh: zh.unwrap_or_default(),
            });
            number += 1;
        }
        Ok(examples)
    }
}

pub fn parse_note_markdown(text: &str) -> Result<NoteImportContent, String> {
    let mut template = Template {
        rest: text.strip_prefix('\u{feff}').unwrap_or(text),
    };
    if template.heading() != Some("# Lexique note v1") {
        return Err("Expected # Lexique note v1 template identifier".into());
    }
    let (mut headword, mut notes, mut examples) = (None, None, None);
    while let Some(heading) = template.heading() {
        match heading {
            "## headword" if headword.is_none() => headword = Some(template.text()?),
            "## notes" if notes.is_none() => {
                notes = Some(
                    if template.peek().is_some_and(|line| line.starts_with('`')) {
                        template.text()?
                    } else {
                        String::new()
                    },
                );
            }
            "## examples" if examples.is_none() => examples = Some(template.examples()?),
            "## headword" | "## notes" | "## examples" => {
                return Err(format!("Duplicate section: {heading}"));
            }
            _ => return Err(format!("Unknown template structure: {heading}")),
        }
    }
    let mut content = NoteImportContent {
        headword: headword.ok_or("Missing required ## headword section")?,
        notes,
        examples,
    };
    content
        .create_request("markdown-validation")
        .map_err(|error| error.to_string())?;
    if let Some(examples) = &mut content.examples {
        examples.retain(|example| !example.fr.is_empty() || !example.zh.is_empty());
    }
    Ok(content)
}

#[cfg(test)]
#[path = "note_markdown_tests.rs"]
mod tests;
