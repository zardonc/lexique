use lexique_storage::{
    DigestEntry, Reference, SaveRequest, Snapshot, add_review, initialize, review_cards, save,
};
use rusqlite::Connection;

fn saved_word(connection: &mut Connection) -> i64 {
    let request = SaveRequest {
        operation: "create-review-word".into(),
        headword: "école".into(),
        notes: "my notes".into(),
        id: None,
        revision: None,
    };
    let snapshot = Snapshot {
        reference: Reference {
            dictionary: "FreeDict".into(),
            lookup_key: "école".into(),
            version: "1".into(),
            attribution: "source".into(),
            license: "CC-BY-SA-3.0".into(),
            source_url: "https://example.org".into(),
            license_url: "https://example.org".into(),
            sha256: "a".repeat(64),
        },
        digest: vec![DigestEntry {
            pronunciations: vec![],
            grammar: vec![],
            translation: Some("学校".into()),
        }],
    };
    save(connection, &request, Some(snapshot))
        .unwrap()
        .record
        .id
}

#[test]
fn adding_a_saved_word_creates_two_cards_once_without_changing_the_record() {
    let mut connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(include_str!("../migrations/0001_initial.sql"))
        .unwrap();
    connection
        .execute_batch(include_str!("../migrations/0002_vocabulary.sql"))
        .unwrap();
    let id = saved_word(&mut connection);
    let original = lexique_storage::get(&connection, id).unwrap();
    assert_eq!(initialize(&connection).unwrap(), 3);
    assert!(review_cards(&connection, id).unwrap().is_empty());
    let first = add_review(&mut connection, id).unwrap();
    assert_eq!(first.len(), 2);
    assert_eq!(
        first
            .iter()
            .map(|card| card.kind.as_str())
            .collect::<Vec<_>>(),
        ["recognition", "listening"]
    );
    let retry = add_review(&mut connection, id).unwrap();
    assert_eq!(first, retry);
    assert_eq!(
        serde_json::to_value(lexique_storage::get(&connection, id).unwrap()).unwrap(),
        serde_json::to_value(original).unwrap()
    );
}

#[test]
fn adding_an_unknown_word_fails_without_creating_cards() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    assert!(add_review(&mut connection, 123).is_err());
    assert!(review_cards(&connection, 123).unwrap().is_empty());
}

#[test]
fn failed_commit_leaves_no_cards_and_retry_creates_both() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved_word(&mut connection);
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(add_review(&mut connection, id).is_err());
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    assert!(review_cards(&connection, id).unwrap().is_empty());
    assert_eq!(add_review(&mut connection, id).unwrap().len(), 2);
}

#[test]
fn cards_keep_their_identity_after_reopening_the_database() {
    let path = std::env::temp_dir().join(format!(
        "lexique-review-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let cards;
    let id;
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        id = saved_word(&mut connection);
        cards = add_review(&mut connection, id).unwrap();
    }
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        assert_eq!(review_cards(&connection, id).unwrap(), cards);
        assert_eq!(add_review(&mut connection, id).unwrap(), cards);
    }
    std::fs::remove_file(path).unwrap();
}
