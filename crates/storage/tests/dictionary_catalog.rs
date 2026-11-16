use lexique_storage::{initialize, read_dictionary_catalog};
use rusqlite::Connection;
use serde_json::json;

#[test]
fn adding_the_core_preserves_existing_order_and_all_fifteen_custom_slots() {
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let fresh = read_dictionary_catalog(&connection).unwrap();
    assert_eq!(
        fresh
            .entries
            .iter()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>(),
        ["builtin-core", "builtin", "builtin-fra-eng"]
    );
    let mut old = vec![json!({"id": "builtin-fra-eng", "title": "English", "content_hash": null})];
    old.extend((1..=15).rev().map(|id| {
        json!({"id": format!("mdx-{id}"),
        "title": format!("User {id}"), "content_hash": "b".repeat(64)})
    }));
    old.push(json!({"id": "builtin", "title": "Chinese", "content_hash": null}));
    connection
        .execute(
            "INSERT INTO app_metadata(key,value) VALUES('dictionary-settings',?1)",
            [json!({"next_id": 16, "entries": old}).to_string()],
        )
        .unwrap();
    let upgraded = read_dictionary_catalog(&connection).unwrap();
    assert_eq!(upgraded.next_id, 16);
    assert_eq!(upgraded.entries.len(), 18);
    assert_eq!(
        serde_json::to_value(&upgraded.entries[..17]).unwrap(),
        json!(old)
    );
    assert_eq!(upgraded.entries[17].id, "builtin-core");
}
