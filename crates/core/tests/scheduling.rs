use lexique_core::scheduling::schedule_json;
use serde_json::{Value, json};

#[test]
fn vancouver_2026_permanent_daylight_time_keeps_the_learning_boundary_at_four() {
    use lexique_core::scheduling::{learning_day_for, next_learning_day_at};
    let zone = "America/Vancouver";
    // 2026-11-05 04:00 at UTC-07:00, after B.C. stopped changing clocks.
    let boundary = 1_793_876_400;
    assert_eq!(next_learning_day_at(boundary - 1, zone).unwrap(), boundary);
    assert_eq!(learning_day_for(boundary - 1, zone).unwrap(), "2026-11-04");
    assert_eq!(learning_day_for(boundary, zone).unwrap(), "2026-11-05");
    // Existing timestamps retain their identity; pre-change winter still used UTC-08.
    assert_eq!(
        next_learning_day_at(1_767_609_599, zone).unwrap(),
        1_767_614_400
    );
}

#[test]
fn timezone_data_update_keeps_replay_versions_explicit() {
    use lexique_core::scheduling::{next_learning_day_at, timezone_database};
    assert_eq!(timezone_database(), "2026f");
    // Cuba's cancelled November 2026 fallback is newer than the host ICU 2026e oracle.
    assert_eq!(
        next_learning_day_at(1_793_861_203, "America/Havana").unwrap(),
        1_793_865_600
    );
    let request = json!({
        "card": {"phase": "new", "memory": null, "last_review_at": null, "due_at": null},
        "rating": "easy", "now": 1793861203_i64, "timezone": "America/Vancouver",
        "timezone_database": "2025b"
    });
    let result: Value = serde_json::from_str(&schedule_json(&request.to_string())).unwrap();
    assert!(
        result["Err"]
            .as_str()
            .unwrap()
            .contains("unsupported timezone database")
    );
}

#[test]
fn first_again_updates_memory_and_waits_a_full_minute() {
    let request = json!({
        "card": {"phase": "new", "memory": null, "last_review_at": null, "due_at": null},
        "rating": "again", "now": 1792929600_i64, "timezone": "Europe/Paris"
    });
    let result: Value = serde_json::from_str(&schedule_json(&request.to_string())).unwrap();
    assert_eq!(result["Ok"]["card"]["phase"], "learning_first");
    assert_eq!(result["Ok"]["step_seconds"], 60);
    assert_eq!(result["Ok"]["card"]["due_at"], 1792929660_i64);
    assert_eq!(result["Ok"]["card"]["memory"]["stability"], json!(0.212));
}

#[test]
fn rejects_invalid_retention_and_clock_rollback_even_within_one_learning_day() {
    let mut request = json!({
        "card": {"phase": "new", "memory": null, "last_review_at": null, "due_at": null},
        "rating": "easy", "now": 1792929600_i64, "timezone": "Europe/Paris",
        "retention_percent": 69
    });
    let result: Value = serde_json::from_str(&schedule_json(&request.to_string())).unwrap();
    assert!(result["Err"].is_string(), "{result}");
    request["retention_percent"] = json!(90);
    request["latest_review_at"] = json!(1792929601_i64);
    let result: Value = serde_json::from_str(&schedule_json(&request.to_string())).unwrap();
    assert!(result["Err"].is_string(), "{result}");
}

fn assert_subset(actual: &Value, expected: &Value, path: &str) {
    if let Some(object) = expected.as_object() {
        for (key, value) in object {
            assert_subset(&actual[key], value, &format!("{path}.{key}"));
        }
    } else if path.contains(".memory.") {
        assert!(
            (actual.as_f64().unwrap() - expected.as_f64().unwrap()).abs() < 0.000001,
            "{path}: {actual} != {expected}"
        );
    } else {
        assert_eq!(actual, expected, "{path}");
    }
}

#[test]
fn fixed_vectors_cover_all_steps_calendar_boundaries_and_invalid_inputs() {
    let vectors: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/scheduling.json")).unwrap();
    for vector in vectors {
        let name = vector["name"].as_str().unwrap();
        let result: Value =
            serde_json::from_str(&schedule_json(&vector["request"].to_string())).unwrap();
        if let Some(error) = vector["error"].as_str() {
            assert!(
                result["Err"]
                    .as_str()
                    .is_some_and(|message| message.contains(error)),
                "{name}: {result}"
            );
        } else {
            assert!(!result["Ok"].is_null(), "{name}: {result}");
            assert_subset(&result["Ok"], &vector["expected"], name);
        }
    }
}

#[test]
fn every_rating_updates_memory_and_historical_requests_replay_without_latest_settings() {
    let mut request = json!({
        "card": {"phase": "new", "memory": null, "last_review_at": null, "due_at": null},
        "rating": "again", "now": 1792929600_i64, "timezone": "Europe/Paris", "retention_percent": 70
    });
    for rating in ["again", "good", "good", "again", "hard", "good", "easy"] {
        request["rating"] = json!(rating);
        let original = request.clone();
        let result: Value = serde_json::from_str(&schedule_json(&request.to_string())).unwrap();
        let scheduled = &result["Ok"];
        assert!(!scheduled.is_null(), "{result}");
        assert_ne!(scheduled["card"]["memory"], request["card"]["memory"]);
        request["model"] = scheduled["model"].clone();
        assert_eq!(
            schedule_json(&request.to_string()),
            schedule_json(&original.to_string())
        );
        let mut latest_settings = request.clone();
        latest_settings["retention_percent"] = json!(99);
        let changed: Value =
            serde_json::from_str(&schedule_json(&latest_settings.to_string())).unwrap();
        assert_eq!(changed["Ok"]["card"]["memory"], scheduled["card"]["memory"]);
        assert_eq!(scheduled["retention_percent"], 70);
        request["card"] = scheduled["card"].clone();
        request["now"] = scheduled["card"]["due_at"].clone();
    }
}

#[test]
fn malformed_or_oversized_json_remains_an_explicit_failure() {
    for input in ["not json".to_owned(), " ".repeat(16_385)] {
        let result: Value = serde_json::from_str(&schedule_json(&input)).unwrap();
        assert!(result["Err"].is_string());
    }
}

#[test]
fn replay_rejects_the_unpatched_platform_math_revision() {
    let mut request = json!({
        "card": {"phase": "new", "memory": null, "last_review_at": null, "due_at": null},
        "rating": "easy", "now": 1792929600_i64, "timezone": "Europe/Paris"
    });
    let result: Value = serde_json::from_str(&schedule_json(&request.to_string())).unwrap();
    assert_eq!(
        result["Ok"]["model"]["version"],
        "fsrs-rs/6.6.2+libm-0.2.16"
    );
    request["model"] = result["Ok"]["model"].clone();
    request["model"]["version"] = json!("fsrs-rs/6.6.2");
    let rejected: Value = serde_json::from_str(&schedule_json(&request.to_string())).unwrap();
    assert!(
        rejected["Err"]
            .as_str()
            .unwrap()
            .contains("unsupported historical model")
    );
}
