use std::path::PathBuf;

use lexique_dictionary::conjugation::{Cell, Lookup, Table, lookup};
use sha2::{Digest, Sha256};

fn resource() -> (PathBuf, String) {
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/desktop-conjugation-v6-cli");
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("report.json")).unwrap()).unwrap();
    (
        directory.join("conjugation.sqlite3"),
        report["sqlite_sha256"].as_str().unwrap().to_owned(),
    )
}

fn query(key: &str) -> Lookup {
    let (path, hash) = resource();
    lookup(&path, key, &hash).unwrap()
}

fn table<'a>(result: &'a Lookup, lemma: &str) -> &'a Table {
    result
        .tables
        .iter()
        .find(|table| table.lemma == lemma)
        .unwrap()
}

fn matched<'a>(table: &'a Table, key: &str) -> Vec<&'a Cell> {
    table
        .cells
        .iter()
        .filter(|cell| {
            cell.form == key || cell.expressions.iter().any(|expression| expression == key)
        })
        .collect()
}

#[test]
fn compact_models_preserve_ambiguity_variants_nullable_agreement_and_order() {
    let result = query("suis");
    assert_eq!(result.query, "suis");
    assert_eq!(
        result
            .tables
            .iter()
            .map(|table| table.lemma.as_str())
            .collect::<Vec<_>>(),
        ["suivre", "être"]
    );
    let follow = matched(table(&result, "suivre"), "suis");
    assert_eq!(
        follow
            .iter()
            .map(|cell| (
                cell.tense.as_deref(),
                cell.person.as_deref(),
                cell.number.as_deref(),
                cell.source_line
            ))
            .collect::<Vec<_>>(),
        [
            (Some("P"), Some("1"), Some("s"), 484864),
            (Some("P"), Some("2"), Some("s"), 484864),
            (Some("Y"), Some("2"), Some("s"), 484865)
        ]
    );
    assert_eq!(matched(table(&result, "être"), "suis").len(), 1);
    let result = query("paie");
    let payer = table(&result, "payer");
    assert_eq!(matched(payer, "paie").len(), 5);
    assert_eq!(
        payer
            .cells
            .iter()
            .filter(|cell| cell.tense.as_deref() == Some("P")
                && cell.person.as_deref() == Some("1")
                && cell.number.as_deref() == Some("s"))
            .map(|cell| cell.form.as_str())
            .collect::<Vec<_>>(),
        ["paie", "paye"]
    );
    for cells in payer.cells.windows(2) {
        let order = |cell: &Cell| {
            (
                cell.tense.clone(),
                cell.person.clone(),
                cell.number.clone(),
                cell.gender.clone(),
                cell.source_line,
            )
        };
        assert!(order(&cells[0]) <= order(&cells[1]));
    }
    let result = query("abstenues");
    assert_eq!(
        result
            .tables
            .iter()
            .map(|table| table.lemma.as_str())
            .collect::<Vec<_>>(),
        ["abstenir", "s’abstenir"]
    );
    assert_eq!(
        table(&result, "abstenir").verb["ordinary_status"],
        "not_applicable"
    );
    assert_eq!(
        table(&result, "s’abstenir").verb["pronominal"]["agreement"],
        "subject"
    );
    for identity in &result.tables {
        let cell = matched(identity, "abstenues")[0];
        assert_eq!(
            (
                cell.source_line,
                cell.person.as_deref(),
                cell.number.as_deref(),
                cell.gender.as_deref()
            ),
            (54573, None, Some("p"), Some("f"))
        );
    }
    let result = query("été");
    assert!(
        table(&result, "être")
            .cells
            .iter()
            .any(|cell| cell.form == "été"
                && cell.tense.as_deref() == Some("K")
                && cell.person.is_none()
                && cell.number.is_none()
                && cell.gender.is_none())
    );
    let result = query("aimer");
    assert_eq!(
        table(&result, "aimer").meanings["senses"][0]["glosses"],
        serde_json::json!(["love"])
    );
    assert_eq!(
        table(&result, "aimer").meanings["senses"][1]["glosses"],
        serde_json::json!(["appreciate", "like"])
    );
    let result = query("voilà");
    let source_only = table(&result, "voilà");
    assert_eq!(source_only.verb["group"], serde_json::Value::Null);
    assert!(source_only.cells.iter().any(|cell| cell.tense.is_none()));
}

#[test]
fn ordinary_constructions_verify_full_queries_and_keep_source_anchors() {
    for (key, lemma) in [
        ("avons aimé", "aimer"),
        ("je suis allée", "aller"),
        ("en ayant aimé", "aimer"),
        ("que nous ayons aimé", "aimer"),
        ("je suis chue", "choir"),
        ("ayons chu", "choir"),
        ("aie occis", "occire"),
        ("j’habite", "habiter"),
        ("je hais", "haïr"),
        ("je yorubaïse", "yorubaïser"),
    ] {
        let result = query(key);
        let selected = table(&result, lemma);
        assert!(!matched(selected, key).is_empty(), "{key}");
        for cell in matched(selected, key)
            .into_iter()
            .filter(|cell| matches!(cell.tense.as_deref(), Some("PC" | "GP" | "GPP" | "YP")))
        {
            assert_eq!(
                cell.source_line,
                selected
                    .cells
                    .iter()
                    .map(|cell| cell.source_line)
                    .min()
                    .unwrap(),
                "{key}"
            );
        }
    }
    let result = query("aie occis");
    assert_eq!(
        matched(table(&result, "occire"), "aie occis")
            .iter()
            .map(|cell| cell.tense.as_deref())
            .collect::<Vec<_>>(),
        [Some("SP"), Some("YP")]
    );
    let result = query("j’ai pu");
    assert_eq!(
        result
            .tables
            .iter()
            .map(|table| table.lemma.as_str())
            .collect::<Vec<_>>(),
        ["paître", "pouvoir"]
    );
    assert_eq!(
        table(&result, "paître").verb["auxiliaries"][0]["use"],
        "falconry"
    );
    let result = query("aille");
    let cell = table(&result, "aller")
        .cells
        .iter()
        .find(|cell| {
            cell.form == "aille"
                && cell.tense.as_deref() == Some("S")
                && cell.person.as_deref() == Some("3")
        })
        .unwrap();
    assert_eq!(
        cell.expressions,
        [
            "il aille",
            "qu’il aille",
            "elle aille",
            "qu’elle aille",
            "on aille",
            "qu’on aille"
        ]
    );
    for key in [
        "nous avez aimé",
        "en ayant vraiment aimé",
        "j’ai parvenu",
        "je suis avenu",
        "j’hais",
        "je habite",
        "j’yorubaïse",
        "' OR 1=1 --",
    ] {
        assert!(query(key).tables.is_empty(), "{key}");
    }
}

#[test]
fn pronominal_constructions_preserve_agreement_restrictions_and_elision() {
    for (key, lemma) in [
        ("je me lave", "se laver"),
        ("lave-toi", "se laver"),
        ("en se lavant", "se laver"),
        ("elles se sont abstenues", "s’abstenir"),
        ("elles se sont lavées", "se laver"),
        ("elles se sont parlé", "se parler"),
        ("elle s’est arrogé", "s’arroger"),
        ("va-t’en", "s’en aller"),
        ("je m’en suis allé", "s’en aller"),
        ("on s’est abstenues", "s’abstenir"),
        ("nous nous sommes écrites", "s’écrire"),
        ("je m’habille", "s’habiller"),
        ("je me hais", "se haïr"),
        ("il s’agit", "s’agir"),
        ("on s’entraide", "s’entraider"),
    ] {
        let result = query(key);
        assert_eq!(
            result
                .tables
                .iter()
                .map(|table| table.lemma.as_str())
                .collect::<Vec<_>>(),
            [lemma],
            "{key}"
        );
        assert!(!matched(&result.tables[0], key).is_empty(), "{key}");
    }
    let result = query("elles se sont abstenues");
    let selected = table(&result, "s’abstenir");
    let compound = matched(selected, "elles se sont abstenues")[0];
    let participle = selected
        .cells
        .iter()
        .find(|cell| cell.tense.as_deref() == Some("K") && cell.form == "abstenues")
        .unwrap();
    assert_eq!(compound.source_line, participle.source_line);
    assert_eq!(compound.source_line, 54573);
    let result = query("on s’est abstenues");
    assert_eq!(
        matched(&result.tables[0], "on s’est abstenues")[0].expressions,
        ["on s’est abstenues"]
    );
    let result = query("elles se sont lavées");
    assert_eq!(
        result.tables[0].verb["pronominal"]["agreement"],
        "contextual"
    );
    assert_eq!(
        matched(&result.tables[0], "elles se sont lavées")[0].expressions,
        ["ils se sont lavées", "elles se sont lavées"]
    );
    let result = query("elles se sont parlé");
    assert_eq!(
        result.tables[0].verb["pronominal"]["agreement"],
        "invariable"
    );
    assert!(
        matched(&result.tables[0], "elles se sont parlé")
            .iter()
            .all(|cell| cell.gender.is_none())
    );
    for key in [
        "elles se sont abstenus",
        "elles se sont parlées",
        "je se lave",
        "elle se suis lavée",
        "je m’agis",
        "je m’entraide",
        "il s’est abstenues",
        "s’étant agis",
        "s’en découdre",
    ] {
        assert!(query(key).tables.is_empty(), "{key}");
    }
}

#[test]
fn normalized_queries_preserve_identities_forms_and_whole_expression_matches() {
    let result = query(" SOMMES ");
    assert_eq!(result.query, " SOMMES ");
    assert_eq!(
        result
            .tables
            .iter()
            .map(|table| table.lemma.as_str())
            .collect::<Vec<_>>(),
        ["sommer", "être"]
    );
    assert_eq!(matched(table(&result, "sommer"), "sommes").len(), 2);
    assert_eq!(matched(table(&result, "être"), "sommes").len(), 1);
    let result = query(" ABSTENUES ");
    assert_eq!(
        result
            .tables
            .iter()
            .map(|table| table.lemma.as_str())
            .collect::<Vec<_>>(),
        ["abstenir", "s’abstenir"]
    );
    for (entered, lemma, expression) in [
        ("\u{a0}E\u{302}TRE\u{202f}", "être", "être"),
        (" E\u{301}TE\u{301} ", "être", "été"),
        ("  J'AI  AIME\u{301}  ", "aimer", "j’ai aimé"),
        (" QUE  J'AIME ", "aimer", "que j’aime"),
        (
            " ELLES\u{a0}SE   SONT ABSTENUES ",
            "s’abstenir",
            "elles se sont abstenues",
        ),
        (" VA-T'EN ", "s’en aller", "va-t’en"),
        (" S'ABSTENIR ", "s’abstenir", "s’abstenir"),
        ("V’LÀ", "voilà", "v'là"),
    ] {
        let result = query(entered);
        assert_eq!(result.query, entered);
        assert_eq!(
            result
                .tables
                .iter()
                .map(|table| table.lemma.as_str())
                .collect::<Vec<_>>(),
            [lemma],
            "{entered}"
        );
        assert!(
            matched(&result.tables[0], expression)
                .iter()
                .any(|cell| { cell.expressions.iter().any(|value| value == expression) }),
            "{entered}"
        );
    }
    let result = query(" J'AI PU ");
    assert_eq!(
        result
            .tables
            .iter()
            .map(|table| table.lemma.as_str())
            .collect::<Vec<_>>(),
        ["paître", "pouvoir"]
    );
    for entered in [
        "ETE",
        " J'AI AIME ",
        " JE AVONS AIMÉ ",
        " JE SE LAVE ",
        "ELLES SE SONT ABSTENUS",
        " AIMÉ AVONS ",
    ] {
        let result = query(entered);
        assert_eq!(result.query, entered);
        assert!(result.tables.is_empty(), "{entered}");
    }
}

#[test]
fn missing_corrupt_and_unsupported_resources_fail_without_writes() {
    let (source, expected) = resource();
    let directory = source
        .parent()
        .unwrap()
        .join(format!("conjugation-lookup-test-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("conjugation.sqlite3");
    assert_eq!(
        lookup(&path, "aimer", &expected).unwrap_err().code,
        "missing"
    );
    for key in [
        "",
        " ",
        "\u{a0}\u{202f}",
        "a\n",
        "a\u{7f}",
        "a\u{85}",
        &"a".repeat(513),
        &"é".repeat(257),
    ] {
        assert_eq!(
            lookup(&path, key, &expected).unwrap_err().code,
            "invalid_query"
        );
    }
    std::fs::write(&path, b"corrupt").unwrap();
    assert_eq!(
        lookup(&path, "aimer", &expected).unwrap_err().code,
        "corrupt"
    );
    std::fs::copy(&source, &path).unwrap();
    let before = std::fs::read(&path).unwrap();
    assert!(!lookup(&path, "aimer", &expected).unwrap().tables.is_empty());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 5).unwrap();
    drop(connection);
    let hash = format!("{:x}", Sha256::digest(std::fs::read(&path).unwrap()));
    let error = lookup(&path, "aimer", &hash).unwrap_err();
    assert_eq!(error.code, "corrupt");
    assert!(
        error
            .message
            .contains("Unsupported conjugation resource format")
    );
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 6).unwrap();
    connection
        .execute(
            "UPDATE verbs SET information = ?1 WHERE lemma = ?2",
            (&b"invalid zlib"[..], "aimer"),
        )
        .unwrap();
    drop(connection);
    let hash = format!("{:x}", Sha256::digest(std::fs::read(&path).unwrap()));
    let error = lookup(&path, "aimer", &hash).unwrap_err();
    assert_eq!(error.code, "corrupt");
    assert!(error.message.contains("decompress conjugation metadata"));
    std::fs::remove_dir_all(directory).unwrap();
}
