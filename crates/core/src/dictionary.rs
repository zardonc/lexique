use serde::{Deserialize, Serialize};

pub const CORE_DICTIONARY_ID: &str = "builtin-core";

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NounGender {
    M,
    F,
    Mf,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FieldSource {
    pub title: String,
    pub version: String,
    pub attribution: String,
    pub source_url: String,
    pub license: String,
    pub license_url: String,
    pub span: String,
    pub modification: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FieldSources {
    pub definition_fr: FieldSource,
    pub ipa: FieldSource,
    pub meaning_en: FieldSource,
    pub meaning_zh: FieldSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gender: Option<FieldSource>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CoreSense {
    pub sense_id: String,
    pub frozen_row: u32,
    pub headword: String,
    pub source_headword: String,
    pub pos: String,
    pub usage: String,
    pub definition_fr: String,
    pub ipa: String,
    pub meaning_en: String,
    pub meaning_zh: String,
    pub field_sources: FieldSources,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gender: Option<NounGender>,
}

fn text(name: &str, value: &str, maximum: usize) -> Result<(), String> {
    if value.trim().is_empty()
        || value.chars().count() > maximum
        || value.chars().any(char::is_control)
    {
        return Err(format!(
            "Core {name} must contain 1–{maximum} characters without control characters"
        ));
    }
    Ok(())
}

fn web_url(value: &str) -> bool {
    value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
        .is_some_and(|rest| !rest.is_empty() && !rest.starts_with(['/', '?', '#']))
        && !value.chars().any(char::is_whitespace)
}

impl CoreSense {
    pub fn validate(&self) -> Result<(), String> {
        if (self.gender.is_some() && self.pos != "NOM")
            || self.gender.is_some() != self.field_sources.gender.is_some()
        {
            return Err("Noun gender requires a NOM sense and matching provenance".into());
        }
        if !(1..=4_087).contains(&self.frozen_row)
            || !matches!(
                self.pos.as_str(),
                "AUX"
                    | "NOM"
                    | "VER"
                    | "ADV"
                    | "ADJ"
                    | "ONO"
                    | "PRO:per"
                    | "PRE"
                    | "ART:def"
                    | "CON"
                    | "ADJ:num"
                    | "ADJ:pos"
                    | "ADJ:int"
                    | "ART:ind"
                    | "PRO:ind"
                    | "PRO:int"
                    | "PRO:rel"
                    | "PRO:dem"
                    | "PRO:pos"
                    | "ADJ:ind"
                    | "ADJ:dem"
            )
        {
            return Err("Invalid frozen core group".into());
        }
        text("sense identity", &self.sense_id, 128)?;
        text("headword", &self.headword, 200)?;
        text("source headword", &self.source_headword, 200)?;
        text("IPA", &self.ipa, 200)?;
        for (name, value) in [
            ("use", &self.usage),
            ("French definition", &self.definition_fr),
            ("English meaning", &self.meaning_en),
            ("Chinese meaning", &self.meaning_zh),
        ] {
            text(name, value, 20_000)?;
        }
        for source in [
            &self.field_sources.definition_fr,
            &self.field_sources.ipa,
            &self.field_sources.meaning_en,
            &self.field_sources.meaning_zh,
        ]
        .into_iter()
        .chain(self.field_sources.gender.iter())
        {
            for value in [
                &source.title,
                &source.version,
                &source.attribution,
                &source.source_url,
                &source.license,
                &source.license_url,
                &source.span,
                &source.modification,
            ] {
                text("field provenance", value, 20_000)?;
            }
            if !web_url(&source.source_url) || !web_url(&source.license_url) {
                return Err("Core provenance links must be HTTP or HTTPS URLs".into());
            }
        }
        Ok(())
    }
}
