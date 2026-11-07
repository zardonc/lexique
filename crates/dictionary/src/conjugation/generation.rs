//! Construction rules shared with the admitted resource converters.
use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use super::Cell;
use crate::{DictionaryError, failure};

pub(super) type Construction = HashMap<String, HashMap<String, HashMap<String, String>>>;

#[derive(Deserialize)]
pub(super) struct Pattern {
    pub cut: u32,
    pub suffix: String,
    raw_tag: String,
    tense: Option<String>,
    person: Option<String>,
    number: Option<String>,
    gender: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct Verb {
    ordinary_status: String,
    auxiliary_status: String,
    auxiliaries: Vec<Auxiliary>,
    usage_labels: Vec<String>,
    correction: Option<Correction>,
    pronominal: Option<Pronominal>,
}

#[derive(Deserialize)]
struct Auxiliary {
    lemma: String,
    r#use: String,
}

#[derive(Default, Deserialize)]
struct Correction {
    persons: Option<Vec<String>>,
    #[serde(default)]
    compound_imperative_persons: Vec<String>,
}

#[derive(Deserialize)]
struct Pronominal {
    agreement: Agreement,
    persons: Vec<String>,
    with_en: bool,
    reciprocal: bool,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Agreement {
    Invariable,
    Subject,
    Contextual,
}

const PERSONS: [&str; 6] = ["1s", "2s", "3s", "1p", "2p", "3p"];
pub(super) const COMPOUND: [(&str, &str); 11] = [
    ("PC", "P"),
    ("PQP", "I"),
    ("PA", "J"),
    ("FA", "F"),
    ("CP", "C"),
    ("SP", "S"),
    ("SQP", "T"),
    ("YP", "Y"),
    ("WP", "W"),
    ("KP", "G"),
    ("GPP", "G"),
];
type CellIdentity = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

pub(super) fn ordinary(
    lemma: &str,
    model: &[Pattern],
    anchors: &[u8],
    verb: &Verb,
    h_aspire: bool,
    construction: &Construction,
) -> Result<Vec<Cell>, DictionaryError> {
    if model.is_empty() || anchors.len() != model.len() * 4 {
        return Err(failure(
            "corrupt",
            "Invalid conjugation source anchors",
            lemma,
        ));
    }
    let boundaries: Vec<_> = lemma
        .char_indices()
        .map(|(index, _)| index)
        .chain([lemma.len()])
        .collect();
    let mut cells = Vec::with_capacity(model.len());
    for (pattern, anchor) in model.iter().zip(anchors.as_chunks::<4>().0) {
        let remaining = (boundaries.len() - 1)
            .checked_sub(pattern.cut as usize)
            .ok_or_else(|| failure("corrupt", "Invalid conjugation model cut", lemma))?;
        cells.push(Cell {
            source_line: u32::from_le_bytes(*anchor),
            form: lemma[..boundaries[remaining]].to_owned() + &pattern.suffix,
            lemma: lemma.to_owned(),
            raw_tag: pattern.raw_tag.clone(),
            tense: pattern.tense.clone(),
            person: pattern.person.clone(),
            number: pattern.number.clone(),
            gender: pattern.gender.clone(),
            expressions: Vec::new(),
            coloring: Default::default(),
        });
    }
    let admitted = verb.ordinary_status == "supported";
    let impersonal = verb.usage_labels.iter().any(|label| label == "impersonal");
    if admitted {
        let correction = verb.correction.as_ref();
        let allowed: Vec<&str> = correction
            .and_then(|correction| correction.persons.as_ref())
            .map(|persons| persons.iter().map(String::as_str).collect())
            .unwrap_or_else(|| {
                if impersonal {
                    vec!["3s"]
                } else {
                    PERSONS.to_vec()
                }
            });
        let first = cells[0].source_line;
        let mut seen: HashSet<_> = cells.iter().map(identity).collect();
        let mut generated = Vec::new();
        for cell in cells
            .iter()
            .filter(|cell| cell.tense.as_deref() == Some("G"))
        {
            add(
                &mut generated,
                &mut seen,
                constructed(
                    first,
                    format!("en {}", cell.form),
                    lemma,
                    "ordinary/v1/gerund".to_owned(),
                    "GP",
                    None,
                    None,
                ),
            );
        }
        if verb.auxiliary_status == "supported" {
            for auxiliary in &verb.auxiliaries {
                for (tense, base) in COMPOUND {
                    for pn in persons(base, &PERSONS) {
                        if pn.is_some_and(|pn| !allowed.contains(&pn)) {
                            continue;
                        }
                        if base == "Y"
                            && !correction.is_some_and(|correction| {
                                pn.is_some_and(|pn| {
                                    correction
                                        .compound_imperative_persons
                                        .iter()
                                        .any(|person| person == pn)
                                })
                            })
                            && !has_imperative(&cells, pn)
                        {
                            continue;
                        }
                        let Some(prefix) = source_form(construction, &auxiliary.lemma, base, pn)
                        else {
                            continue;
                        };
                        for pp in cells
                            .iter()
                            .filter(|cell| cell.tense.as_deref() == Some("K"))
                        {
                            let number = pp.number.as_deref();
                            let gender = pp.gender.as_deref();
                            if auxiliary.lemma == "avoir"
                                && (gender.is_some_and(|gender| gender != "m")
                                    || number.is_some_and(|number| number != "s"))
                            {
                                continue;
                            }
                            if auxiliary.lemma == "être"
                                && (gender.is_none()
                                    || number.is_none()
                                    || pn.is_some_and(|pn| pn != "2p" && number != pn.get(1..)))
                            {
                                continue;
                            }
                            add(
                                &mut generated,
                                &mut seen,
                                constructed(
                                    first,
                                    format!(
                                        "{}{prefix} {}",
                                        if tense == "GPP" { "en " } else { "" },
                                        pp.form
                                    ),
                                    lemma,
                                    format!("ordinary/v1/{}/{}", auxiliary.lemma, auxiliary.r#use),
                                    tense,
                                    pn,
                                    if auxiliary.lemma == "être" {
                                        gender
                                    } else {
                                        None
                                    },
                                ),
                            );
                        }
                    }
                }
            }
        }
        cells.extend(generated);
    }
    for cell in &mut cells {
        cell.expressions = if admitted {
            expressions(cell, cell.gender.as_deref(), impersonal, h_aspire)
        } else {
            vec![cell.form.clone()]
        };
    }
    Ok(cells)
}

pub(super) fn pronominal(
    lemma: &str,
    original: &[Cell],
    verb: &Verb,
    h_aspire: bool,
    construction: &Construction,
) -> Result<Vec<Cell>, DictionaryError> {
    let info = verb
        .pronominal
        .as_ref()
        .ok_or_else(|| failure("corrupt", "Missing pronominal construction metadata", lemma))?;
    let mut cells = Vec::new();
    let mut seen = HashSet::new();
    for row in original {
        let Some(tense) = row.tense.as_deref() else {
            continue;
        };
        if COMPOUND.iter().any(|(code, _)| *code == tense) || tense == "GP" {
            continue;
        }
        let pn = person_number(row);
        if !matches!(tense, "W" | "G" | "K") && !PERSONS.contains(&pn.as_str()) {
            continue;
        }
        if row.person.is_some() && !info.persons.contains(&pn) {
            continue;
        }
        if tense == "Y" && !["2s", "1p", "2p"].contains(&pn.as_str()) {
            continue;
        }
        if tense == "K" && info.agreement == Agreement::Invariable && !invariable(row) {
            continue;
        }
        let form = if tense == "K" {
            row.form.clone()
        } else {
            reflexive(&row.form, Some(&pn), tense == "Y", info.with_en, h_aspire)?
        };
        let mut cell = Cell {
            source_line: row.source_line,
            form,
            lemma: lemma.to_owned(),
            raw_tag: pronominal_tag(info),
            tense: row.tense.clone(),
            person: row.person.clone(),
            number: row.number.clone(),
            gender: row.gender.clone(),
            expressions: Vec::new(),
            coloring: Default::default(),
        };
        pronominal_expressions(&mut cell, info, false, false);
        let gerund = (tense == "G").then(|| format!("en {}", cell.form));
        add(&mut cells, &mut seen, cell);
        if let Some(form) = gerund {
            add(
                &mut cells,
                &mut seen,
                constructed(
                    row.source_line,
                    form,
                    lemma,
                    pronominal_tag(info),
                    "GP",
                    None,
                    None,
                ),
            );
        }
    }
    let persons: Vec<_> = info.persons.iter().map(String::as_str).collect();
    for (tense, base) in COMPOUND {
        for pn in self::persons(base, &persons) {
            if base == "Y" && !has_imperative(&cells, pn) {
                continue;
            }
            let Some(prefix) = source_form(construction, "être", base, pn) else {
                continue;
            };
            for pp in original
                .iter()
                .filter(|cell| cell.tense.as_deref() == Some("K"))
            {
                let number = pp.number.as_deref();
                if info.agreement == Agreement::Invariable {
                    if !invariable(pp) {
                        continue;
                    }
                } else if pp.gender.is_none()
                    || number.is_none()
                    || info.agreement == Agreement::Subject
                        && pn.is_some_and(|pn| !["2p", "3s"].contains(&pn) && number != pn.get(1..))
                {
                    continue;
                }
                let verbal = reflexive(prefix, pn, base == "Y", info.with_en, false)?;
                let mut cell = constructed(
                    pp.source_line,
                    format!(
                        "{}{verbal} {}",
                        if tense == "GPP" { "en " } else { "" },
                        pp.form
                    ),
                    lemma,
                    pronominal_tag(info),
                    tense,
                    pn,
                    if info.agreement == Agreement::Invariable {
                        None
                    } else {
                        pp.gender.as_deref()
                    },
                );
                pronominal_expressions(
                    &mut cell,
                    info,
                    info.agreement == Agreement::Contextual,
                    info.agreement == Agreement::Subject && pn == Some("3s") && number == Some("p"),
                );
                add(&mut cells, &mut seen, cell);
            }
        }
    }
    for cell in &mut cells {
        if cell.tense.as_deref() == Some("GP") {
            cell.expressions = vec![cell.form.clone()];
        }
    }
    Ok(cells)
}

fn identity(cell: &Cell) -> CellIdentity {
    (
        cell.form.clone(),
        cell.tense.clone(),
        cell.person.clone(),
        cell.number.clone(),
        cell.gender.clone(),
    )
}

fn add(cells: &mut Vec<Cell>, seen: &mut HashSet<CellIdentity>, cell: Cell) {
    if seen.insert(identity(&cell)) {
        cells.push(cell)
    }
}

fn constructed(
    line: u32,
    form: String,
    lemma: &str,
    raw_tag: String,
    tense: &str,
    pn: Option<&str>,
    gender: Option<&str>,
) -> Cell {
    Cell {
        source_line: line,
        form,
        lemma: lemma.to_owned(),
        raw_tag,
        tense: Some(tense.to_owned()),
        person: pn
            .and_then(|pn| pn.chars().next())
            .map(|person| person.to_string()),
        number: pn
            .and_then(|pn| pn.chars().nth(1))
            .map(|number| number.to_string()),
        gender: gender.map(str::to_owned),
        expressions: Vec::new(),
        coloring: Default::default(),
    }
}

fn person_number(cell: &Cell) -> String {
    format!(
        "{}{}",
        cell.person.as_deref().unwrap_or("None"),
        cell.number.as_deref().unwrap_or("None")
    )
}

fn persons<'a>(base: &str, persons: &[&'a str]) -> Vec<Option<&'a str>> {
    if matches!(base, "W" | "G") {
        vec![None]
    } else {
        persons.iter().copied().map(Some).collect()
    }
}

fn has_imperative(cells: &[Cell], pn: Option<&str>) -> bool {
    cells
        .iter()
        .any(|cell| cell.tense.as_deref() == Some("Y") && pn == Some(person_number(cell).as_str()))
}

fn source_form<'a>(
    construction: &'a Construction,
    auxiliary: &str,
    tense: &str,
    pn: Option<&str>,
) -> Option<&'a str> {
    construction
        .get(auxiliary)?
        .get(tense)?
        .get(pn.unwrap_or(""))
        .map(String::as_str)
        .filter(|form| !form.is_empty())
}

fn invariable(cell: &Cell) -> bool {
    matches!(
        (cell.gender.as_deref(), cell.number.as_deref()),
        (Some("m"), Some("s")) | (None, None)
    )
}

fn pronominal_tag(info: &Pronominal) -> String {
    format!(
        "pronominal/v1/{}",
        match info.agreement {
            Agreement::Invariable => "invariable",
            Agreement::Subject => "subject",
            Agreement::Contextual => "contextual",
        }
    )
}

fn reflexive(
    form: &str,
    pn: Option<&str>,
    imperative: bool,
    with_en: bool,
    h_aspire: bool,
) -> Result<String, DictionaryError> {
    if imperative {
        let suffix = match (pn, with_en) {
            (Some("2s"), false) => "-toi",
            (Some("1p"), false) => "-nous",
            (Some("2p"), false) => "-vous",
            (Some("2s"), true) => "-t’en",
            (Some("1p"), true) => "-nous-en",
            (Some("2p"), true) => "-vous-en",
            _ => {
                return Err(failure(
                    "corrupt",
                    "Invalid pronominal imperative person",
                    pn.unwrap_or("unspecified"),
                ));
            }
        };
        return Ok(form.to_owned() + suffix);
    }
    let pronoun = match pn {
        Some("1s") => "me",
        Some("2s") => "te",
        Some("1p") => "nous",
        Some("2p") => "vous",
        _ => "se",
    };
    let en = if with_en { "en " } else { "" };
    Ok(
        if matches!(pronoun, "me" | "te" | "se") && (with_en || elides(form, h_aspire)) {
            format!("{}’{en}{form}", &pronoun[..1])
        } else {
            format!("{pronoun} {en}{form}")
        },
    )
}

fn pronominal_expressions(cell: &mut Cell, info: &Pronominal, context: bool, collective: bool) {
    let mut result = expressions(
        cell,
        if context {
            None
        } else {
            cell.gender.as_deref()
        },
        false,
        false,
    );
    if collective
        || info.reciprocal
            && cell.person.as_deref() == Some("3")
            && cell.number.as_deref() == Some("s")
    {
        result
            .retain(|expression| expression.starts_with("on ") || expression.starts_with("qu’on "));
    }
    if info.persons == ["3s"] && cell.person.is_some() {
        result
            .retain(|expression| expression.starts_with("il ") || expression.starts_with("qu’il "));
    }
    cell.expressions = result;
}

fn expressions(cell: &Cell, gender: Option<&str>, impersonal: bool, h_aspire: bool) -> Vec<String> {
    let tense = cell.tense.as_deref();
    let pn = person_number(cell);
    if matches!(
        tense,
        Some("Y" | "YP" | "W" | "WP" | "G" | "K" | "KP" | "GP" | "GPP")
    ) || impersonal && pn != "3s"
    {
        return vec![cell.form.clone()];
    }
    let subjects: &[&str] = if impersonal && pn == "3s" {
        &["il"]
    } else {
        match pn.as_str() {
            "1s" => &["je"],
            "2s" => &["tu"],
            "3s" => &["il", "elle", "on"],
            "1p" => &["nous"],
            "2p" => &["vous"],
            "3p" => &["ils", "elles"],
            _ => &[],
        }
    };
    let mut result = Vec::new();
    for subject in subjects {
        if gender == Some("f") && matches!(*subject, "il" | "ils")
            || gender == Some("m") && matches!(*subject, "elle" | "elles")
        {
            continue;
        }
        let verbal = if *subject == "je" && elides(&cell.form, h_aspire) {
            format!("j’{}", cell.form)
        } else {
            format!("{subject} {}", cell.form)
        };
        result.push(verbal.clone());
        if matches!(tense, Some("S" | "T" | "SP" | "SQP")) {
            result.push(format!(
                "{}{verbal}",
                if ["il", "elle", "on"]
                    .iter()
                    .any(|prefix| verbal.starts_with(prefix))
                {
                    "qu’"
                } else {
                    "que "
                }
            ));
        }
    }
    result
}

fn elides(form: &str, h_aspire: bool) -> bool {
    let mut letters = form.chars();
    let first = letters.next();
    first.is_some_and(|first| "aàâeéèêëiîïoôœuùûü".contains(first))
        || first == Some('h') && !h_aspire
        || first == Some('y')
            && !letters
                .next()
                .is_some_and(|second| "aeiouy".contains(second))
}
