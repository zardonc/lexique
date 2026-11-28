//! User-imported FLELex grades; never part of a saved vocabulary record.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{DictionaryError, validate_query};

pub const MAX_BYTES: usize = 5 * 1024 * 1024;
const HEADER: &str =
    "word\ttag\tfreq_A1\tfreq_A2\tfreq_B1\tfreq_B2\tfreq_C1\tfreq_C2\tfreq_total\tlevel";

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    A1,
    A2,
    B1,
    B2,
    C1,
    C2,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sense {
    pub sense_id: String,
    pub pos: String,
    pub source_headword: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub key: String,
    pub sense: Option<Sense>,
}

#[derive(Debug, Serialize)]
pub struct Entry {
    pub key: String,
    pub sense_id: Option<String>,
    pub level: Option<Level>,
}

#[derive(Deserialize)]
struct Alignment {
    pos_map: BTreeMap<String, Vec<String>>,
    aliases: BTreeMap<String, Vec<String>>,
    excluded_sense_ids: BTreeSet<String>,
}

pub struct Dataset {
    rows: BTreeMap<(String, String), Level>,
    alignment: Alignment,
}

fn failure(message: impl Into<String>) -> DictionaryError {
    DictionaryError {
        code: "invalid_query",
        message: message.into(),
    }
}

impl Dataset {
    pub fn parse(bytes: &[u8]) -> Result<Self, DictionaryError> {
        if bytes.is_empty() || bytes.len() > MAX_BYTES {
            return Err(failure("FLELex TSV must be non-empty and at most 5 MiB"));
        }
        let text = std::str::from_utf8(bytes)
            .map_err(|error| failure(format!("FLELex requires UTF-8: {error}")))?;
        let mut lines = text.lines();
        if lines.next() != Some(HEADER) {
            return Err(failure(
                "Expected the exact 10-column FleLex_TT_Beacco.tsv header",
            ));
        }
        let mut rows = BTreeMap::new();
        for (index, line) in lines.enumerate() {
            if line.is_empty() {
                continue;
            }
            let columns: Vec<_> = line.split('\t').collect();
            let invalid = || {
                failure(format!(
                    "Invalid FLELex row {}: require word, tag, seven finite non-negative frequencies and A1–C2",
                    index + 2
                ))
            };
            if columns.len() != 10 {
                return Err(invalid());
            }
            for value in &columns[..2] {
                if value.trim() != *value || value.is_empty() || value.chars().any(char::is_control)
                {
                    return Err(invalid());
                }
            }
            for value in &columns[2..9] {
                if value
                    .parse::<f64>()
                    .ok()
                    .is_none_or(|number| !number.is_finite() || number < 0.0)
                {
                    return Err(invalid());
                }
            }
            let level = match columns[9] {
                "A1" => Level::A1,
                "A2" => Level::A2,
                "B1" => Level::B1,
                "B2" => Level::B2,
                "C1" => Level::C1,
                "C2" => Level::C2,
                _ => return Err(invalid()),
            };
            if rows
                .insert((columns[0].into(), columns[1].into()), level)
                .is_some()
            {
                return Err(failure(format!(
                    "Duplicate FLELex (word, tag) at row {}",
                    index + 2
                )));
            }
        }
        if rows.is_empty() {
            return Err(failure("FLELex TSV contains no data rows"));
        }
        let alignment =
            serde_json::from_str(include_str!("../../../resources/cefr-alignment.json"))
                .map_err(|error| failure(format!("Invalid bundled CEFR alignment: {error}")))?;
        Ok(Self { rows, alignment })
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn lookup(&self, query: &Query) -> Option<Level> {
        let mut words = vec![query.key.as_str()];
        if let Some(sense) = &query.sense {
            if self.alignment.excluded_sense_ids.contains(&sense.sense_id) {
                return None;
            }
            words.push(&sense.source_headword);
        }
        let seeds = words.clone();
        for word in seeds {
            if let Some(aliases) = self.alignment.aliases.get(word) {
                words.extend(aliases.iter().map(String::as_str));
            }
        }
        for word in words {
            let levels: BTreeSet<_> = self
                .rows
                .range((word.to_owned(), String::new())..)
                .take_while(|((candidate, _), _)| candidate == word)
                .filter_map(|((_, tag), level)| {
                    query
                        .sense
                        .as_ref()
                        .is_none_or(|sense| {
                            self.alignment
                                .pos_map
                                .get(&sense.pos)
                                .is_some_and(|tags| tags.contains(tag))
                        })
                        .then_some(*level)
                })
                .collect();
            if !levels.is_empty() {
                return (levels.len() == 1).then(|| *levels.first().unwrap());
            }
        }
        None
    }
}

pub fn validate_queries(queries: &[Query]) -> Result<(), DictionaryError> {
    if queries.is_empty() || queries.len() > 200 {
        return Err(failure("CEFR lookup requires 1–200 queries"));
    }
    for query in queries {
        validate_query(&query.key)?;
        if let Some(sense) = &query.sense {
            validate_query(&sense.source_headword)?;
            if sense.sense_id.trim().is_empty()
                || sense.sense_id.len() > 128
                || sense.sense_id.chars().any(char::is_control)
                || sense.pos.is_empty()
                || sense.pos.len() > 16
                || sense.pos.chars().any(char::is_control)
            {
                return Err(failure("Invalid CEFR sense identity or POS"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn file(rows: &str) -> Vec<u8> {
        format!("{HEADER}\n{rows}").into_bytes()
    }
    #[test]
    fn exact_spelling_precedes_a_differently_graded_source_spelling() {
        for (key, source, pos) in [
            ("enceinte", "enceint", "ADJ"),
            ("clef", "clé", "NOM"),
            ("cieux", "ciel", "NOM"),
        ] {
            let data = Dataset::parse(&file(&format!(
                "{key}\t{pos}\t0\t0\t0\t0\t0\t0\t0\tA1\n{source}\t{pos}\t0\t0\t0\t0\t0\t0\t0\tC1\n"
            )))
            .unwrap();
            let mut query = Query {
                key: key.into(),
                sense: Some(Sense {
                    sense_id: "selected-sense".into(),
                    pos: pos.into(),
                    source_headword: source.into(),
                }),
            };
            query.sense.as_mut().unwrap().source_headword = key.into();
            assert_eq!(data.lookup(&query), Some(Level::A1));
            query.sense.as_mut().unwrap().source_headword = source.into();
            assert_eq!(data.lookup(&query), Some(Level::A1), "{key}");
            query.key = "missing-spelling".into();
            assert_eq!(data.lookup(&query), Some(Level::C1));
            query.sense.as_mut().unwrap().pos = "VER".into();
            assert_eq!(data.lookup(&query), None);
        }
    }
    #[test]
    fn validates_beacco_and_preserves_pos_and_sense_boundaries() {
        let bytes = file(
            "coeur\tNOM\t0\t0\t0\t0\t0\t0\t0\tB1\nferme\tNOM\t0\t0\t0\t0\t0\t0\t0\tB2\nferme\tADJ\t0\t0\t0\t0\t0\t0\t0\tA2\npar\tNOM\t0\t0\t0\t0\t0\t0\t0\tC1\n",
        );
        let data = Dataset::parse(&bytes).unwrap();
        assert_eq!(data.row_count(), 4);
        assert!(
            Dataset::parse(
                &String::from_utf8(bytes.clone())
                    .unwrap()
                    .replace('\n', "\r\n")
                    .into_bytes()
            )
            .is_ok()
        );
        let query = |key: &str, pos: &str, id: &str| Query {
            key: key.into(),
            sense: Some(Sense {
                sense_id: id.into(),
                pos: pos.into(),
                source_headword: key.into(),
            }),
        };
        assert_eq!(data.lookup(&query("cœur", "NOM", "heart")), Some(Level::B1));
        assert_eq!(data.lookup(&query("ferme", "NOM", "farm")), Some(Level::B2));
        assert_eq!(data.lookup(&query("ferme", "VER", "close")), None);
        assert_eq!(
            data.lookup(&Query {
                key: "ferme".into(),
                sense: None
            }),
            None
        );
        assert_eq!(
            data.lookup(&query("par", "NOM", "par-noun-golf-standard-strokes")),
            None
        );
        assert!(validate_queries(&[query("ferme", "NOM", "farm")]).is_ok());
        assert!(validate_queries(&[]).is_err());
    }
    #[test]
    fn rejects_corrupt_or_non_beacco_inputs() {
        for bytes in [
            vec![],
            vec![b'x'; MAX_BYTES + 1],
            vec![0xff],
            b"word\ttag\tlevel\n".to_vec(),
            file("x\tNOM\t0\t0\t0\t0\t0\t0\tNaN\tA1"),
            file("x\tNOM\t-1\t0\t0\t0\t0\t0\t0\tA1"),
            file("x\tNOM\t0\t0\t0\t0\t0\t0\t0\tA0"),
            file("x\tNOM\t0\t0\t0\t0\t0\t0\t0\tA1\nx\tNOM\t0\t0\t0\t0\t0\t0\t0\tA2"),
            file(""),
        ] {
            assert!(Dataset::parse(&bytes).is_err());
        }
    }
}
