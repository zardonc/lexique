use chrono::{DateTime, Datelike, Days, NaiveDate, TimeZone, Timelike};
use chrono_tz::Tz;
use fsrs::{DEFAULT_PARAMETERS, FSRS, MemoryState};
use serde::{Deserialize, Serialize};

const MODEL_VERSION: &str = "fsrs-rs/6.6.2+libm-0.2.16";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    New,
    LearningFirst,
    LearningLast,
    Review,
    Relearning,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Rating {
    Again,
    Hard,
    Good,
    Easy,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Memory {
    pub stability: f32,
    pub difficulty: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Card {
    pub phase: Phase,
    pub memory: Option<Memory>,
    pub last_review_at: Option<i64>,
    pub due_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub version: String,
    pub parameters: Vec<f32>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            version: MODEL_VERSION.into(),
            parameters: DEFAULT_PARAMETERS.to_vec(),
        }
    }
}

fn default_retention() -> u8 {
    90
}

pub fn timezone_database() -> String {
    chrono_tz::IANA_TZDB_VERSION.into()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleRequest {
    pub card: Card,
    pub rating: Rating,
    pub now: i64,
    pub timezone: String,
    #[serde(default = "timezone_database")]
    pub timezone_database: String,
    pub latest_review_at: Option<i64>,
    #[serde(default = "default_retention")]
    pub retention_percent: u8,
    #[serde(default)]
    pub model: Model,
}

#[derive(Debug, Serialize)]
pub struct Scheduled {
    pub card: Card,
    pub elapsed_days: u32,
    pub step_seconds: Option<u32>,
    pub interval_days: Option<u32>,
    pub model: Model,
    pub retention_percent: u8,
    pub timezone_database: String,
}

fn learning_day(timestamp: i64, zone: Tz) -> Result<NaiveDate, String> {
    let utc = DateTime::from_timestamp(timestamp, 0).ok_or("timestamp out of range")?;
    if !(1..=9999).contains(&utc.year()) {
        return Err("timestamp year must be between 1 and 9999".into());
    }
    let local = utc.with_timezone(&zone);
    if local.hour() < 4 {
        local
            .date_naive()
            .pred_opt()
            .ok_or("date out of range".into())
    } else {
        Ok(local.date_naive())
    }
}

pub fn learning_day_for(timestamp: i64, timezone: &str) -> Result<String, String> {
    let zone: Tz = timezone.parse().map_err(|_| "unknown timezone")?;
    Ok(learning_day(timestamp, zone)?.to_string())
}

pub fn next_learning_day_at(timestamp: i64, timezone: &str) -> Result<i64, String> {
    let zone: Tz = timezone.parse().map_err(|_| "unknown timezone")?;
    let date = learning_day(timestamp, zone)?
        .succ_opt()
        .ok_or("date out of range")?;
    Ok(zone
        .from_local_datetime(&date.and_hms_opt(4, 0, 0).ok_or("invalid time")?)
        .single()
        .ok_or("next 04:00 is ambiguous or nonexistent")?
        .timestamp())
}

/// Uses the recorded portable-math revision for reproducible scheduling.
pub fn schedule(request: ScheduleRequest) -> Result<Scheduled, String> {
    if request.timezone_database != chrono_tz::IANA_TZDB_VERSION {
        return Err("unsupported timezone database; use the recorded core build for replay".into());
    }
    if !(70..=99).contains(&request.retention_percent) {
        return Err("retention must be an integer percentage between 70 and 99".into());
    }
    if request.model.version != MODEL_VERSION
        || request.model.parameters.as_slice() != DEFAULT_PARAMETERS
    {
        return Err("unsupported historical model version or parameters".into());
    }
    for previous in [request.latest_review_at, request.card.last_review_at]
        .into_iter()
        .flatten()
    {
        learning_day(previous, chrono_tz::UTC)?;
        if request.now < previous {
            return Err("clock moved backwards; check device time before rating".into());
        }
    }
    match request.card.phase {
        Phase::New
            if request.card.memory.is_none()
                && request.card.last_review_at.is_none()
                && request.card.due_at.is_none() => {}
        Phase::New => return Err("new card must not contain learned state".into()),
        _ if request.card.memory.is_none()
            || request.card.last_review_at.is_none()
            || request.card.due_at.is_none() =>
        {
            return Err("learned card requires memory and timestamps".into());
        }
        _ => {}
    }
    if let Some(memory) = request.card.memory
        && (!memory.stability.is_finite()
            || !(0.001..=36_500.0).contains(&memory.stability)
            || !memory.difficulty.is_finite()
            || !(1.0..=10.0).contains(&memory.difficulty))
    {
        return Err("invalid memory state".into());
    }
    let zone: Tz = request.timezone.parse().map_err(|_| "unknown timezone")?;
    let today = learning_day(request.now, zone)?;
    if let Some(due) = request.card.due_at {
        learning_day(due, zone)?;
        if due < request.card.last_review_at.unwrap() {
            return Err("due time precedes previous rating".into());
        }
        if request.now < due {
            return Err("card is not due".into());
        }
    }
    let elapsed_days = match request.card.last_review_at {
        Some(previous) => u32::try_from((today - learning_day(previous, zone)?).num_days())
            .map_err(|_| "clock moved backwards")?,
        None => 0,
    };
    let current = request.card.memory.map(|memory| MemoryState {
        stability: memory.stability,
        difficulty: memory.difficulty,
    });
    let states = FSRS::default()
        .next_states(
            current,
            f32::from(request.retention_percent) / 100.0,
            elapsed_days,
        )
        .map_err(|error| format!("FSRS: {error}"))?;
    let next = match request.rating {
        Rating::Again => states.again,
        Rating::Hard => states.hard,
        Rating::Good => states.good,
        Rating::Easy => states.easy,
    };
    if !next.interval.is_finite()
        || !next.memory.stability.is_finite()
        || !next.memory.difficulty.is_finite()
    {
        return Err("FSRS produced a non-finite result".into());
    }
    let (phase, step_seconds) = match (request.card.phase, request.rating) {
        (Phase::New | Phase::LearningFirst, Rating::Again) => (Phase::LearningFirst, Some(60)),
        (Phase::New | Phase::LearningFirst, Rating::Hard) => (Phase::LearningFirst, Some(330)),
        (Phase::New | Phase::LearningFirst, Rating::Good) => (Phase::LearningLast, Some(600)),
        (Phase::LearningLast, Rating::Again) => (Phase::LearningFirst, Some(60)),
        (Phase::LearningLast, Rating::Hard) => (Phase::LearningLast, Some(600)),
        (Phase::Review | Phase::Relearning, Rating::Again) => (Phase::Relearning, Some(600)),
        (Phase::Relearning, Rating::Hard) => (Phase::Relearning, Some(900)),
        _ => (Phase::Review, None),
    };
    let interval_days = step_seconds
        .is_none()
        .then(|| next.interval.round().clamp(1.0, 36_500.0) as u32);
    let due_at = if let Some(seconds) = step_seconds {
        request
            .now
            .checked_add(i64::from(seconds))
            .ok_or("due timestamp overflow")?
    } else {
        let date = today
            .checked_add_days(Days::new(u64::from(interval_days.unwrap())))
            .ok_or("due date out of range")?;
        zone.from_local_datetime(&date.and_hms_opt(4, 0, 0).unwrap())
            .single()
            .ok_or("target 04:00 is ambiguous or nonexistent")?
            .timestamp()
    };
    learning_day(due_at, zone)?;
    Ok(Scheduled {
        card: Card {
            phase,
            memory: Some(Memory {
                stability: next.memory.stability,
                difficulty: next.memory.difficulty,
            }),
            last_review_at: Some(request.now),
            due_at: Some(due_at),
        },
        elapsed_days,
        step_seconds,
        interval_days,
        model: request.model,
        retention_percent: request.retention_percent,
        timezone_database: request.timezone_database,
    })
}

pub fn schedule_json(input: &str) -> String {
    let result = if input.len() > 16_384 {
        Err("request exceeds 16384 bytes".into())
    } else {
        serde_json::from_str::<ScheduleRequest>(input)
            .map_err(|error| format!("invalid request: {error}"))
            .and_then(schedule)
    };
    serde_json::to_string(&result).expect("schedule output contains only finite numbers")
}
