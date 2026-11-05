use lexique_storage::{StorageError, initialize};
use rusqlite::Connection;

#[test]
fn initialization_is_repeatable_and_preserves_existing_values() {
    let connection = Connection::open_in_memory().unwrap();
    assert_eq!(
        initialize(&connection).unwrap(),
        lexique_core::DATABASE_SCHEMA_VERSION
    );
    connection
        .execute(
            "INSERT INTO app_metadata (key, value) VALUES (?1, ?2)",
            ["example", "keep"],
        )
        .unwrap();
    assert_eq!(
        initialize(&connection).unwrap(),
        lexique_core::DATABASE_SCHEMA_VERSION
    );
    let value: String = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = ?1",
            ["example"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(value, "keep");
}

#[test]
fn future_schema_is_rejected_without_rewriting_it() {
    let connection = Connection::open_in_memory().unwrap();
    connection.pragma_update(None, "user_version", 99).unwrap();
    assert!(matches!(
        initialize(&connection),
        Err(StorageError::FutureSchema(99))
    ));
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 99);
}

#[test]
fn conflicting_schema_returns_an_error_and_rolls_back() {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute("CREATE TABLE app_metadata (unexpected TEXT)", [])
        .unwrap();
    assert!(initialize(&connection).is_err());
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 0);
}

#[test]
fn version_one_metadata_survives_incremental_migration_and_failure_rolls_back() {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(include_str!("../migrations/0001_initial.sql"))
        .unwrap();
    connection
        .execute(
            "INSERT INTO app_metadata (key, value) VALUES ('keep', 'original')",
            [],
        )
        .unwrap();
    connection
        .execute("CREATE TABLE vocabulary (unexpected TEXT)", [])
        .unwrap();
    assert!(initialize(&connection).is_err());
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
            .unwrap(),
        1
    );
    connection.execute("DROP TABLE vocabulary", []).unwrap();
    assert_eq!(
        initialize(&connection).unwrap(),
        lexique_core::DATABASE_SCHEMA_VERSION
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT value FROM app_metadata WHERE key = 'keep'",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "original"
    );
}

#[test]
fn version_two_migration_preserves_vocabulary_and_rolls_back_on_conflict() {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(include_str!("../migrations/0001_initial.sql"))
        .unwrap();
    connection
        .execute_batch(include_str!("../migrations/0002_vocabulary.sql"))
        .unwrap();
    connection.execute("INSERT INTO vocabulary (headword, snapshot, notes, created_at, updated_at) VALUES ('test', '{}', 'keep', 1, 1)", []).unwrap();
    connection
        .execute("CREATE TABLE review_cards (unexpected TEXT)", [])
        .unwrap();
    assert!(initialize(&connection).is_err());
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
            .unwrap(),
        2
    );
    connection.execute("DROP TABLE review_cards", []).unwrap();
    assert_eq!(
        initialize(&connection).unwrap(),
        lexique_core::DATABASE_SCHEMA_VERSION
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT notes FROM vocabulary WHERE headword = 'test'",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "keep"
    );
    assert!(
        lexique_storage::review_cards(&connection, 1)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn version_three_rating_migration_rolls_back_on_conflict() {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(include_str!("../migrations/0001_initial.sql"))
        .unwrap();
    connection
        .execute_batch(include_str!("../migrations/0002_vocabulary.sql"))
        .unwrap();
    connection
        .execute_batch(include_str!("../migrations/0003_review.sql"))
        .unwrap();
    connection
        .execute("CREATE TABLE review_ratings (unexpected TEXT)", [])
        .unwrap();
    assert!(initialize(&connection).is_err());
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
            .unwrap(),
        3
    );
    let created: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name = 'review_progress')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!created);
}
