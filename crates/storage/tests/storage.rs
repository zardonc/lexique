use lexique_storage::{StorageError, initialize};
use rusqlite::Connection;

#[test]
fn initialization_is_repeatable_and_preserves_existing_values() {
    let connection = Connection::open_in_memory().unwrap();
    assert_eq!(initialize(&connection).unwrap(), 1);
    connection
        .execute(
            "INSERT INTO app_metadata (key, value) VALUES (?1, ?2)",
            ["example", "keep"],
        )
        .unwrap();
    assert_eq!(initialize(&connection).unwrap(), 1);
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
