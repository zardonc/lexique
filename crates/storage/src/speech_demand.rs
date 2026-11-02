//! Read-only demand planning seam. Sentence synthesis remains subject to M3 admission.
use std::collections::HashSet;

use crate::{SpeechSpec, StorageError};

pub struct AudioCandidate {
    pub card_id: i64,
    pub word: SpeechSpec,
    pub sentence: Option<SpeechSpec>,
    pub due_at: Option<i64>,
    pub paused: bool,
    pub skipped: bool,
    pub quota_available: bool,
    pub deferred: bool,
}

pub struct PlannedAudio<'a> {
    pub semantic_key: String,
    pub spec: &'a SpeechSpec,
    pub priority: u8,
}

/// Preserve supplied review order, select current plus at most three candidates,
/// and deduplicate shared word audio. No schedule, quota or played state is written.
pub fn plan_speech_window(
    candidates: &[AudioCandidate],
    current_id: Option<i64>,
    now: i64,
) -> Result<Vec<PlannedAudio<'_>>, StorageError> {
    let horizon = now
        .checked_add(604800)
        .ok_or_else(|| StorageError::Invalid("Invalid prefetch clock".into()))?;
    let mut selected: Vec<_> = candidates
        .iter()
        .filter(|candidate| {
            !candidate.paused
                && !candidate.skipped
                && candidate.quota_available
                && !candidate.deferred
                && candidate.due_at.is_none_or(|due| due <= horizon)
        })
        .collect();
    if let Some(index) = selected
        .iter()
        .position(|candidate| Some(candidate.card_id) == current_id)
    {
        selected.drain(..index);
    }
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for candidate in selected.into_iter().take(4) {
        for spec in std::iter::once(&candidate.word).chain(candidate.sentence.iter()) {
            let semantic_key = spec.key()?;
            if seen.insert(semantic_key.clone()) {
                result.push(PlannedAudio {
                    semantic_key,
                    spec,
                    priority: if Some(candidate.card_id) == current_id {
                        0
                    } else {
                        1
                    },
                });
            }
        }
    }
    Ok(result)
}

pub fn usable_through(generated_at: i64, required_until: i64) -> bool {
    generated_at
        .checked_add(604800)
        .is_some_and(|expiry| required_until < expiry)
}
