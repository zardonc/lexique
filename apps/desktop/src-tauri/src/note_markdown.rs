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
                files: None,
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
    safe_markdown_filename(&record.headword, &record.id.to_string(), 80)
}

pub fn grammar_markdown_filename(record: &lexique_storage::GrammarExportRecord) -> String {
    safe_markdown_filename(
        &record.title,
        &format!("{}-{}", record.exchange_id, record.id),
        40,
    )
}

fn safe_markdown_filename(title: &str, suffix: &str, max_chars: usize) -> String {
    let readable: String = title
        .chars()
        .take(max_chars)
        .map(|ch| {
            if ch.is_control() || "<>:\"/\\|?*".contains(ch) {
                '_'
            } else {
                ch
            }
        })
        .collect();
    let readable = readable.trim_matches([' ', '.']);
    let filename = format!(
        "{}-{}.md",
        if readable.is_empty() {
            "note"
        } else {
            readable
        },
        suffix
    );
    // Windows reserves device basenames even when followed by an extension.
    let basename = filename
        .split('.')
        .next()
        .unwrap_or(&filename)
        .to_ascii_uppercase();
    let numbered_device = basename
        .strip_prefix("COM")
        .or_else(|| basename.strip_prefix("LPT"))
        .is_some_and(|number| {
            ["1", "2", "3", "4", "5", "6", "7", "8", "9", "¹", "²", "³"].contains(&number)
        });
    if matches!(basename.as_str(), "CON" | "PRN" | "AUX" | "NUL") || numbered_device {
        format!("_{filename}")
    } else {
        filename
    }
}

fn field(output: &mut String, heading: &str, text: &str) {
    let length = text.split(|ch| ch != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat((length + 1).max(3));
    output.push_str(&format!("{heading}\n\n{fence}text\n{text}\n{fence}\n\n"));
}

pub fn format_grammar_markdown(record: &lexique_storage::GrammarExportRecord) -> String {
    let mut output = "# Lexique grammar note v1\n\n".to_string();
    field(&mut output, "## exchange_id", &record.exchange_id);
    field(&mut output, "## title", &record.title);
    field(&mut output, "## body", &record.body);
    if !record.question.is_empty() || !record.answer.is_empty() {
        field(&mut output, "## question", &record.question);
        field(&mut output, "## answer", &record.answer);
    }
    output
}

pub fn format_note_markdown(record: &lexique_storage::NoteExportRecord) -> String {
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

pub fn parse_grammar_markdown(text: &str) -> Result<lexique_storage::GrammarImportContent, String> {
    let content = parse_grammar_markdown_fields(text, false)?;
    content
        .validate()
        .map_err(|error| format!("Grammar sections: {error}"))?;
    Ok(content)
}

pub(crate) fn parse_grammar_markdown_fields(
    text: &str,
    allow_missing_title: bool,
) -> Result<lexique_storage::GrammarImportContent, String> {
    let mut template = Template {
        rest: text.strip_prefix('\u{feff}').unwrap_or(text),
    };
    if template.heading() != Some("# Lexique grammar note v1") {
        return Err("Expected # Lexique grammar note v1 template identifier".into());
    }
    let (mut identity, mut title, mut body, mut question, mut answer) =
        (None, None, None, None, None);
    while let Some(heading) = template.heading() {
        let slot = match heading {
            "## exchange_id" => &mut identity,
            "## title" => &mut title,
            "## body" => &mut body,
            "## question" => &mut question,
            "## answer" => &mut answer,
            _ => return Err(format!("Unknown template structure: {heading}")),
        };
        if slot.is_some() {
            return Err(format!("Duplicate section: {heading}"));
        }
        *slot = Some(
            template
                .text()
                .map_err(|error| format!("{heading}: {error}"))?,
        );
    }
    let content = lexique_storage::GrammarImportContent {
        exchange_id: identity,
        title: match title {
            Some(title) => title,
            None if allow_missing_title => String::new(),
            None => return Err("Missing required ## title section".into()),
        },
        body,
        question,
        answer,
    };
    Ok(content)
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
