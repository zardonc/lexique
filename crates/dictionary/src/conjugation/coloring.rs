//! Linguistic spans use admitted ending rules, never storage model cuts or suffixes.
use std::{collections::HashMap, sync::OnceLock};

use serde::Deserialize;
use serde_json::Value;

use super::{Cell, Coloring, generation::COMPOUND};
use crate::{DictionaryError, failure};

type Families = HashMap<String, HashMap<String, Vec<String>>>;

#[derive(Deserialize)]
struct Rules {
    common: Families,
    groups: HashMap<String, Families>,
    lexical: HashMap<String, Families>,
    source_only_participles: HashMap<String, Vec<String>>,
}

fn verbal_word(
    form: &str,
    mut range: [usize; 2],
    pronominal: Option<&Value>,
    imperative: bool,
) -> Result<[usize; 2], DictionaryError> {
    if let Some(info) = pronominal {
        let prefixes: &[&str] = if info["with_en"] == true {
            &["m’en ", "t’en ", "s’en ", "nous en ", "vous en "]
        } else {
            &["m’", "t’", "s’", "me ", "te ", "se ", "nous ", "vous "]
        };
        if let Some(prefix) = prefixes
            .iter()
            .find(|prefix| form[range[0]..range[1]].starts_with(**prefix))
        {
            range[0] += prefix.len();
        }
        if imperative
            && let Some(suffix) = ["-nous-en", "-vous-en", "-t’en", "-toi", "-nous", "-vous"]
                .iter()
                .find(|suffix| form[range[0]..range[1]].ends_with(**suffix))
        {
            range[1] -= suffix.len();
        }
    }
    if range[0] >= range[1] {
        return Err(failure("corrupt", "Empty conjugation verbal part", form));
    }
    Ok(range)
}

fn form_spans(cell: &Cell, verb: &Value) -> Result<(Vec<[usize; 2]>, [usize; 2]), DictionaryError> {
    let tense = cell.tense.as_deref().unwrap_or("");
    let form = &cell.form;
    let start = if matches!(tense, "GP" | "GPP") { 3 } else { 0 };
    if COMPOUND.iter().any(|(code, _)| *code == tense) {
        let split = form
            .rfind(' ')
            .ok_or_else(|| failure("corrupt", "Missing compound main verb", form))?;
        let main = [split + 1, form.len()];
        let auxiliary = verbal_word(form, [start, split], verb.get("pronominal"), tense == "YP")?;
        let verbal = if &form[auxiliary[1]..main[0]] == " " {
            vec![[auxiliary[0], main[1]]]
        } else {
            vec![auxiliary, main]
        };
        Ok((verbal, main))
    } else {
        let main = verbal_word(
            form,
            [start, form.len()],
            if tense == "K" {
                None
            } else {
                verb.get("pronominal")
            },
            tense == "Y",
        )?;
        Ok((vec![main], main))
    }
}

fn inflection<'a>(
    cell: &Cell,
    verb: &Value,
    main: [usize; 2],
    rules: &'a Rules,
) -> Result<Option<&'a str>, DictionaryError> {
    let group = verb["group"].as_u64().unwrap_or(0);
    if verb["group_status"] != "supported" || !matches!(group, 1 | 2) {
        return Ok(None);
    }
    let original = cell.tense.as_deref().unwrap_or("");
    let compound = COMPOUND.iter().any(|(code, _)| *code == original);
    let tense = if compound {
        "K"
    } else if original == "GP" {
        "G"
    } else {
        original
    };
    let base = verb["pronominal"]["base"].as_str().unwrap_or(&cell.lemma);
    let word = &cell.form[main[0]..main[1]];
    if matches!(original, "P" | "I" | "J" | "F" | "C" | "S" | "T" | "Y")
        && (cell.person.is_none() || cell.number.is_none())
        || rules
            .source_only_participles
            .get(base)
            .is_some_and(|forms| forms.iter().any(|form| form == word))
    {
        return Ok(None);
    }
    let family = rules
        .lexical
        .get(base)
        .and_then(|families| families.get(tense))
        .or_else(|| {
            rules
                .groups
                .get(&group.to_string())
                .and_then(|families| families.get(tense))
        })
        .or_else(|| rules.common.get(tense));
    let slot = if compound || matches!(tense, "W" | "G") {
        "_".to_owned()
    } else if tense == "K" {
        format!(
            "{}{}",
            cell.gender.as_deref().unwrap_or("m"),
            cell.number.as_deref().unwrap_or("s")
        )
    } else {
        format!(
            "{}{}",
            cell.person.as_deref().unwrap_or(""),
            cell.number.as_deref().unwrap_or("")
        )
    };
    let endings = family.and_then(|family| family.get(&slot));
    let ending = endings.and_then(|endings| {
        endings
            .iter()
            .find(|ending| word.ends_with(ending.as_str()))
    });
    if endings.is_some_and(|endings| !endings.is_empty()) && ending.is_none() {
        return Err(failure(
            "corrupt",
            "Unvalidated regular conjugation ending",
            format!("{} {original} {}", cell.lemma, cell.form),
        ));
    }
    Ok(ending.map(String::as_str))
}

pub(super) fn apply(cell: &mut Cell, verb: &Value) -> Result<(), DictionaryError> {
    static RULES: OnceLock<Rules> = OnceLock::new();
    let rules = RULES.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../scripts/conjugation_endings.json"))
            .expect("Bundled linguistic ending rules must be valid")
    });
    let (verbal, main) = form_spans(cell, verb)?;
    let ending = inflection(cell, verb, main, rules)?;
    let characters = |position| cell.form[..position].chars().count();
    let main = main.map(characters);
    let verbal: Vec<_> = verbal
        .into_iter()
        .map(|range| range.map(characters))
        .collect();
    let ending = ending.map(|ending| [main[1] - ending.chars().count(), main[1]]);
    for text in std::iter::once(&cell.form).chain(&cell.expressions) {
        let prefix = text.strip_suffix(cell.form.as_str()).ok_or_else(|| {
            failure(
                "corrupt",
                "Conjugation expression does not preserve its form",
                text,
            )
        })?;
        let offset = prefix.chars().count();
        cell.coloring.insert(
            text.clone(),
            Coloring {
                verbal: verbal
                    .iter()
                    .map(|range| range.map(|position| position + offset))
                    .collect(),
                main: main.map(|position| position + offset),
                ending: ending.map(|range| range.map(|position| position + offset)),
            },
        );
    }
    Ok(())
}
