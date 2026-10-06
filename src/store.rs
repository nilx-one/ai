// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

//! Where Avaia's continuity state is kept, as rules rather than as storage.
//!
//! This crate does no I/O. A product supplies the storage behind [`AvaiaStateStore`]; what
//! this module owns is what any such storage must and must not do with the value:
//!
//! - a record is bound to the Avaia it belongs to ([`AvaiaSubject`]), and a record bound to
//!   another subject is never read as this one's;
//! - a v1 record is read and upgraded; a record in a schema newer than this crate is kept as
//!   it is — quarantined, not rewritten — and she runs without it;
//! - a write replaces the record whole, or not at all, and never overwrites a quarantined one.
//!
//! The subject is established by the identity and protocol layer, never by a model: the same
//! Avaia on a new runtime, a new model, or after the model was evicted from a device's cache
//! restores the same state.

use crate::{AVAIA_STATE_SCHEMA_VERSION, AvaiaState};
use serde::{Deserialize, Serialize};

/// The Avaia a stored state belongs to: an identifier the identity layer established.
///
/// Opaque here. It must never be derived from a model, a runtime, a device or a cache: those
/// change while she stays the same.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AvaiaSubject(String);

impl AvaiaSubject {
    /// Creates a non-empty subject.
    ///
    /// # Errors
    /// Returns [`EmptySubject`] for an empty or blank identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, EmptySubject> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(EmptySubject);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AvaiaSubject {
    type Error = EmptySubject;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<AvaiaSubject> for String {
    fn from(value: AvaiaSubject) -> Self {
        value.0
    }
}

/// A subject identifier was empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptySubject;

impl std::fmt::Display for EmptySubject {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an Avaia subject must not be empty")
    }
}

impl std::error::Error for EmptySubject {}

/// What a store holds: one Avaia's state, bound to her.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredAvaiaState {
    pub subject: AvaiaSubject,
    pub state: AvaiaState,
}

/// The storage a product supplies. Only the rules in this module decide what is read from it
/// or written to it; an implementation only keeps bytes.
pub trait AvaiaStateStore {
    /// The storage's own failure: unavailable, unreadable, full.
    type Error;

    /// The record kept for `subject`, as stored, or `None` when there is none.
    ///
    /// # Errors
    /// The storage could not be read.
    fn load(&self, subject: &AvaiaSubject) -> Result<Option<StoredAvaiaState>, Self::Error>;

    /// Replaces the record kept for `record.subject` with `record`, atomically: after a
    /// failure the previous record is still the one kept.
    ///
    /// # Errors
    /// The storage could not be written; nothing changed.
    fn replace(&mut self, record: StoredAvaiaState) -> Result<(), Self::Error>;
}

/// What restoring found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Restored {
    /// Her state, in the current schema.
    State(AvaiaState),
    /// Nothing kept for her yet: she starts from [`AvaiaState::new`].
    Fresh,
    /// Her state is in a schema newer than this crate. It stays as it is in the store, and
    /// this runtime starts her from [`AvaiaState::new`] without writing over it.
    Quarantined { schema_version: u16 },
}

impl Restored {
    /// The state to run with: hers, or a fresh one when there is none this crate can read.
    #[must_use]
    pub fn into_state(self) -> AvaiaState {
        match self {
            Self::State(state) => state,
            Self::Fresh | Self::Quarantined { .. } => AvaiaState::new(),
        }
    }
}

/// Why restoring or persisting was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateStoreError<E> {
    /// The record found belongs to another subject. It is never read as this one's.
    WrongSubject,
    /// The record kept is in a newer schema; writing would destroy what this crate cannot read.
    WouldOverwriteNewer { schema_version: u16 },
    /// Only the current schema is ever written.
    NotCurrentSchema { schema_version: u16 },
    /// The storage itself failed.
    Storage(E),
}

/// Restores `subject`'s state from `store`.
///
/// # Errors
/// [`StateStoreError::WrongSubject`] when the record kept under `subject` is bound to another
/// subject, and [`StateStoreError::Storage`] when the storage could not be read.
pub fn restore<S: AvaiaStateStore>(
    store: &S,
    subject: &AvaiaSubject,
) -> Result<Restored, StateStoreError<S::Error>> {
    let Some(record) = store.load(subject).map_err(StateStoreError::Storage)? else {
        return Ok(Restored::Fresh);
    };
    if &record.subject != subject {
        return Err(StateStoreError::WrongSubject);
    }
    let schema_version = record.state.schema_version;
    Ok(record
        .state
        .upgraded()
        .map_or(Restored::Quarantined { schema_version }, Restored::State))
}

/// Keeps `state` as `subject`'s, replacing what was kept.
///
/// # Errors
/// [`StateStoreError::NotCurrentSchema`] for a state not in the current schema,
/// [`StateStoreError::WrongSubject`] when the record kept under `subject` belongs to another
/// subject, [`StateStoreError::WouldOverwriteNewer`] when it is in a newer schema, and
/// [`StateStoreError::Storage`] when the storage failed. Nothing is written on any of them.
pub fn persist<S: AvaiaStateStore>(
    store: &mut S,
    subject: &AvaiaSubject,
    state: &AvaiaState,
) -> Result<(), StateStoreError<S::Error>> {
    if !state.is_current_schema() {
        return Err(StateStoreError::NotCurrentSchema {
            schema_version: state.schema_version,
        });
    }
    if let Some(kept) = store.load(subject).map_err(StateStoreError::Storage)? {
        if &kept.subject != subject {
            return Err(StateStoreError::WrongSubject);
        }
        if kept.state.schema_version > AVAIA_STATE_SCHEMA_VERSION {
            return Err(StateStoreError::WouldOverwriteNewer {
                schema_version: kept.state.schema_version,
            });
        }
    }
    store
        .replace(StoredAvaiaState {
            subject: subject.clone(),
            state: state.clone(),
        })
        .map_err(StateStoreError::Storage)
}

#[cfg(test)]
mod tests {
    use super::{
        AvaiaStateStore, AvaiaSubject, EmptySubject, Restored, StateStoreError, StoredAvaiaState,
        persist, restore,
    };
    use crate::{
        AVAIA_STATE_SCHEMA_VERSION, AvaiaIntent, AvaiaState, DecisionMenu, MapTargetId, StopAction,
    };
    use std::collections::HashMap;

    /// A store in memory: records by the key they were written under, and a switch that
    /// makes the next write fail, to show a failed write changes nothing.
    #[derive(Default)]
    struct Memory {
        records: HashMap<String, StoredAvaiaState>,
        fail_next_write: bool,
    }

    impl AvaiaStateStore for Memory {
        type Error = &'static str;

        fn load(&self, subject: &AvaiaSubject) -> Result<Option<StoredAvaiaState>, Self::Error> {
            Ok(self.records.get(subject.as_str()).cloned())
        }

        fn replace(&mut self, record: StoredAvaiaState) -> Result<(), Self::Error> {
            if std::mem::take(&mut self.fail_next_write) {
                return Err("disk full");
            }
            self.records
                .insert(record.subject.as_str().to_owned(), record);
            Ok(())
        }
    }

    fn subject(value: &str) -> AvaiaSubject {
        AvaiaSubject::new(value).expect("subject")
    }

    fn walking_to(target: &str) -> AvaiaState {
        let targets = [MapTargetId::new(target).expect("target")];
        let mut state = AvaiaState::new();
        state.decide(
            DecisionMenu::new(&targets, StopAction::Offered)
                .expect("menu")
                .decide(&format!("navigate {target}"))
                .expect("offered"),
        );
        state
    }

    #[test]
    fn a_subject_is_never_empty() {
        assert_eq!(AvaiaSubject::new("  "), Err(EmptySubject));
        assert!(serde_json::from_str::<AvaiaSubject>(r#""""#).is_err());
    }

    #[test]
    fn nothing_kept_is_a_fresh_start() {
        let restored = restore(&Memory::default(), &subject("avaia:1")).expect("restore");
        assert_eq!(restored, Restored::Fresh);
        assert_eq!(restored.into_state(), AvaiaState::new());
    }

    /// The point of the module: she outlives the runtime. A new runtime, a new model, a model
    /// evicted from cache — restoring by the same subject gives back the same state.
    #[test]
    fn what_is_kept_comes_back_for_the_same_subject() {
        let mut store = Memory::default();
        let her = subject("avaia:1");
        let state = walking_to("lake");

        persist(&mut store, &her, &state).expect("persist");
        assert_eq!(restore(&store, &her), Ok(Restored::State(state)));
    }

    #[test]
    fn another_subjects_record_is_never_read_as_hers() {
        let mut store = Memory::default();
        // A storage bug keeps the wrong record under her key.
        store.records.insert(
            "avaia:1".into(),
            StoredAvaiaState {
                subject: subject("avaia:2"),
                state: walking_to("park"),
            },
        );
        let her = subject("avaia:1");

        assert_eq!(restore(&store, &her), Err(StateStoreError::WrongSubject));
        assert_eq!(
            persist(&mut store, &her, &AvaiaState::new()),
            Err(StateStoreError::WrongSubject)
        );
        assert_eq!(store.records["avaia:1"].subject, subject("avaia:2"));
    }

    #[test]
    fn a_v1_record_is_read_and_upgraded() {
        let stored: StoredAvaiaState = serde_json::from_str(
            r#"{
                "subject": "avaia:1",
                "state": {
                    "schema_version": 1,
                    "intent": "Explore",
                    "last_proposal": null,
                    "pause_reason": "OwnerControl"
                }
            }"#,
        )
        .expect("v1 record reads");
        let mut store = Memory::default();
        store.records.insert("avaia:1".into(), stored);

        let Ok(Restored::State(state)) = restore(&store, &subject("avaia:1")) else {
            panic!("a v1 record restores");
        };
        assert!(state.is_current_schema());
        assert_eq!(state.intent, Some(AvaiaIntent::Explore));
    }

    #[test]
    fn a_newer_schema_is_quarantined_and_never_written_over() {
        let her = subject("avaia:1");
        let newer = StoredAvaiaState {
            subject: her.clone(),
            state: AvaiaState {
                schema_version: AVAIA_STATE_SCHEMA_VERSION + 1,
                ..walking_to("lake")
            },
        };
        let mut store = Memory::default();
        store.records.insert("avaia:1".into(), newer.clone());

        let restored = restore(&store, &her).expect("restore");
        assert_eq!(
            restored,
            Restored::Quarantined {
                schema_version: AVAIA_STATE_SCHEMA_VERSION + 1
            }
        );
        // She runs, from fresh, without touching what she cannot read.
        assert_eq!(restored.into_state(), AvaiaState::new());
        assert_eq!(
            persist(&mut store, &her, &AvaiaState::new()),
            Err(StateStoreError::WouldOverwriteNewer {
                schema_version: AVAIA_STATE_SCHEMA_VERSION + 1
            })
        );
        assert_eq!(store.records["avaia:1"], newer);
    }

    #[test]
    fn only_the_current_schema_is_written() {
        let mut store = Memory::default();
        let old = AvaiaState {
            schema_version: 1,
            ..AvaiaState::new()
        };
        assert_eq!(
            persist(&mut store, &subject("avaia:1"), &old),
            Err(StateStoreError::NotCurrentSchema { schema_version: 1 })
        );
        assert!(store.records.is_empty());
    }

    #[test]
    fn a_failed_write_keeps_what_was_kept() {
        let mut store = Memory::default();
        let her = subject("avaia:1");
        let before = walking_to("lake");
        persist(&mut store, &her, &before).expect("persist");

        store.fail_next_write = true;
        assert_eq!(
            persist(&mut store, &her, &walking_to("park")),
            Err(StateStoreError::Storage("disk full"))
        );
        assert_eq!(restore(&store, &her), Ok(Restored::State(before)));
    }
}
