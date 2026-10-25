use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Entry {
    pub ordinal: u32,
    pub key: String,
    pub pronunciations: Vec<String>,
    pub grammar: Vec<String>,
    pub senses: Vec<Sense>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Sense {
    #[serde(default, rename(deserialize = "cit"), skip_serializing)]
    citations: Vec<Citation>,
    #[serde(default, skip_deserializing)]
    pub translations: Vec<String>,
    #[serde(default, rename(deserialize = "def"))]
    pub definitions: Vec<String>,
    #[serde(default, rename(deserialize = "sense"))]
    pub senses: Vec<Sense>,
}

#[derive(Debug, Deserialize)]
struct Citation {
    #[serde(default, rename = "quote")]
    quotes: Vec<String>,
}

#[derive(Deserialize)]
struct Form {
    orth: String,
    #[serde(default)]
    pron: Vec<String>,
}

#[derive(Default, Deserialize)]
struct Grammar {
    #[serde(default)]
    pos: Vec<String>,
    #[serde(default, rename = "gen")]
    gender: Vec<String>,
}

#[derive(Deserialize)]
struct RawEntry {
    form: Form,
    #[serde(default, rename = "gramGrp")]
    grammar: Grammar,
    #[serde(default, rename = "sense")]
    senses: Vec<Sense>,
}

fn collect_translations(sense: &mut Sense) {
    sense.translations = std::mem::take(&mut sense.citations)
        .into_iter()
        .flat_map(|citation| citation.quotes)
        .collect();
    for child in &mut sense.senses {
        collect_translations(child);
    }
}

pub(crate) fn parse(ordinal: u32, key: String, xml: &str) -> Result<Entry, String> {
    let mut raw: RawEntry = quick_xml::de::from_str(xml).map_err(|error| error.to_string())?;
    if raw.form.orth != key {
        return Err(format!(
            "Source spelling differs from key at record {ordinal}"
        ));
    }
    for sense in &mut raw.senses {
        collect_translations(sense);
    }
    Ok(Entry {
        ordinal,
        key,
        pronunciations: raw.form.pron,
        grammar: raw
            .grammar
            .pos
            .into_iter()
            .chain(raw.grammar.gender)
            .collect(),
        senses: raw.senses,
    })
}
