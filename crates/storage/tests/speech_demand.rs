use lexique_storage::*;

fn candidate(id: i64, word: &str) -> AudioCandidate {
    AudioCandidate {
        card_id: id,
        word: SpeechSpec::word(word, SpeechVoice::Female),
        sentence: None,
        due_at: None,
        paused: false,
        skipped: false,
        quota_available: true,
        deferred: false,
    }
}

#[test]
fn current_and_three_card_window_deduplicates_both_card_kinds_without_writes() {
    let mut candidates = vec![
        candidate(1, "école"),
        candidate(2, "école"),
        candidate(3, "bonjour"),
        candidate(4, "rouge"),
        candidate(5, "bleu"),
    ];
    let mut sentence = SpeechSpec::word("Bonjour à tous", SpeechVoice::Female);
    sentence.context = "sentence".into();
    candidates[2].sentence = Some(sentence);
    let planned = plan_speech_window(&candidates, Some(1), 1000).unwrap();
    assert_eq!(planned.len(), 4);
    assert_eq!(planned[0].priority, 0);
    assert_eq!(
        planned
            .iter()
            .map(|item| item.spec.text.as_str())
            .collect::<Vec<_>>(),
        ["école", "bonjour", "Bonjour à tous", "rouge"]
    );
    assert!(
        candidates
            .iter()
            .all(|candidate| candidate.due_at.is_none())
    );
    assert_eq!(
        plan_speech_window(&candidates, Some(3), 1000).unwrap()[0]
            .spec
            .text,
        "bonjour"
    );
    assert_eq!(
        plan_speech_window(&candidates, Some(5), 1000)
            .unwrap()
            .iter()
            .map(|item| item.spec.text.as_str())
            .collect::<Vec<_>>(),
        ["bleu"]
    );
}

#[test]
fn admission_excludes_paused_skipped_quota_deferred_and_beyond_horizon() {
    let mut candidates = (1..=7)
        .map(|id| candidate(id, &format!("mot{id}")))
        .collect::<Vec<_>>();
    candidates[0].paused = true;
    candidates[1].skipped = true;
    candidates[2].quota_available = false;
    candidates[3].deferred = true;
    candidates[4].due_at = Some(605801);
    candidates[5].due_at = Some(605800);
    let planned = plan_speech_window(&candidates, Some(1), 1000).unwrap();
    assert_eq!(
        planned
            .iter()
            .map(|item| item.spec.text.as_str())
            .collect::<Vec<_>>(),
        ["mot6", "mot7"]
    );
    assert!(usable_through(1000, 605799));
    assert!(!usable_through(1000, 605800));
    assert!(!usable_through(1000, 605801));
}

#[test]
fn changed_sentence_and_context_have_distinct_identity_and_partial_examples_do_not_gate_word() {
    let mut candidate = candidate(1, "école");
    assert_eq!(
        plan_speech_window(std::slice::from_ref(&candidate), Some(1), 1000)
            .unwrap()
            .len(),
        1
    );
    let mut sentence = SpeechSpec::word("école", SpeechVoice::Female);
    sentence.context = "sentence".into();
    candidate.sentence = Some(sentence);
    let prior = plan_speech_window(std::slice::from_ref(&candidate), Some(1), 1000).unwrap()[1]
        .semantic_key
        .clone();
    assert_ne!(prior, candidate.word.key().unwrap());
    candidate.sentence.as_mut().unwrap().text = "nouvel exemple".into();
    assert_ne!(
        prior,
        plan_speech_window(&[candidate], Some(1), 1000).unwrap()[1].semantic_key
    );
}
