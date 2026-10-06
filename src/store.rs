// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

//! Where Avaia's continuity state is kept, as rules rather than as storage.
//!
//! This crate does no I/O. A product supplies the storage behind `AvaiaStateStore`. The
//! boundary keeps subject + schema metadata outside the schema-specific payload so a runtime
//! can quarantine a future record without first trying to deserialize it.
//!
//! A write is one guarded storage operation: checking the currently stored subject/schema and
//! replacing the bytes happen atomically inside the adapter. There is no load-then-replace gap
//! in which another context could install a newer schema and have it overwritten.

use crate::{AVAIA_STATE_SCHEMA_VERSION, AvaiaState};
use serde::{Deserialize, Serialize};

const REPLACEABLE_SCHEMAS: &[u16] = &[1, AVAIA_STATE_SCHEMA_VERSION];

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AvaiaSubject(String);

impl AvaiaSubject {
    /// Creates a non-empty subject.
    ///
    /// # Errors
    /// Returns `EmptySubject` for an empty or blank identifier.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptySubject;

impl std::fmt::Display for EmptySubject {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an Avaia subject must not be empty")
    }
}

impl std::error::Error for EmptySubject {}

/// One stored state envelope. Subject and schema version stay outside payload so an unknown
/// future payload never has to be decoded merely to decide that it must be quarantined.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredAvaiaState {
    pub subject: AvaiaSubject,
    pub schema_version: u16,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardedReplace {
    Replaced,
    WrongSubject,
    SchemaNotReplaceable { schema_version: u16 },
}

/// Storage keeps opaque payload bytes and envelope metadata; it does not decide Avaia semantics.
pub trait AvaiaStateStore {
    type Error;

    /// # Errors
    /// The storage could not be read.
    fn load(&self, subject: &AvaiaSubject) -> Result<Option<StoredAvaiaState>, Self::Error>;

    /// Checks the currently stored envelope and replaces it as one atomic storage operation.
    /// The adapter must use one transaction / compare-and-swap equivalent: no record may be
    /// installed between this guard and the replacement.
    ///
    /// # Errors
    /// The storage operation failed; nothing changed.
    fn replace_guarded(
        &mut self,
        subject: &AvaiaSubject,
        replaceable_schema_versions: &[u16],
        record: StoredAvaiaState,
    ) -> Result<GuardedReplace, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Restored {
    State(AvaiaState),
    Fresh,
    Quarantined { schema_version: u16 },
}

impl Restored {
    #[must_use]
    pub fn into_state(self) -> AvaiaState {
        match self {
            Self::State(state) => state,
            Self::Fresh | Self::Quarantined { .. } => AvaiaState::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateStoreError<E> {
    WrongSubject,
    WouldOverwriteUnsupported { schema_version: u16 },
    NotCurrentSchema { schema_version: u16 },
    InvalidPayload { schema_version: u16 },
    EncodeFailed,
    Storage(E),
}

/// Restores state after inspecting subject/schema envelope metadata.
///
/// # Errors
/// Returns `WrongSubject` for a foreign envelope, `InvalidPayload` for malformed bytes of a schema
/// this crate claims to understand, and Storage for an adapter failure.
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
    if !REPLACEABLE_SCHEMAS.contains(&record.schema_version) {
        return Ok(Restored::Quarantined {
            schema_version: record.schema_version,
        });
    }

    let state: AvaiaState =
        serde_json::from_slice(&record.payload).map_err(|_| StateStoreError::InvalidPayload {
            schema_version: record.schema_version,
        })?;
    if state.schema_version != record.schema_version {
        return Err(StateStoreError::InvalidPayload {
            schema_version: record.schema_version,
        });
    }
    state
        .upgraded()
        .map(Restored::State)
        .ok_or(StateStoreError::InvalidPayload {
            schema_version: record.schema_version,
        })
}

/// Persists current state through one atomic guarded replacement, with no preliminary load.
///
/// # Errors
/// Refuses non-current input, foreign/unsupported records, encoding failures, and storage errors.
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
    let payload = serde_json::to_vec(state).map_err(|_| StateStoreError::EncodeFailed)?;
    let record = StoredAvaiaState {
        subject: subject.clone(),
        schema_version: state.schema_version,
        payload,
    };

    match store
        .replace_guarded(subject, REPLACEABLE_SCHEMAS, record)
        .map_err(StateStoreError::Storage)?
    {
        GuardedReplace::Replaced => Ok(()),
        GuardedReplace::WrongSubject => Err(StateStoreError::WrongSubject),
        GuardedReplace::SchemaNotReplaceable { schema_version } => {
            Err(StateStoreError::WouldOverwriteUnsupported { schema_version })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AvaiaStateStore, AvaiaSubject, EmptySubject, GuardedReplace, Restored, StateStoreError,
        StoredAvaiaState, persist, restore,
    };
    use crate::{
        AVAIA_STATE_SCHEMA_VERSION, AvaiaIntent, AvaiaState, DecisionMenu, MapTargetId, StopAction,
    };
    use std::collections::HashMap;

    #[derive(Default)]
    struct Memory {
        records: HashMap<String, StoredAvaiaState>,
        fail_next_write: bool,
        install_before_guard: Option<(String, StoredAvaiaState)>,
    }

    impl AvaiaStateStore for Memory {
        type Error = &'static str;

        fn load(&self, subject: &AvaiaSubject) -> Result<Option<StoredAvaiaState>, Self::Error> {
            Ok(self.records.get(subject.as_str()).cloned())
        }

        fn replace_guarded(
            &mut self,
            subject: &AvaiaSubject,
            replaceable_schema_versions: &[u16],
            record: StoredAvaiaState,
        ) -> Result<GuardedReplace, Self::Error> {
            if let Some((key, concurrent)) = self.install_before_guard.take() {
                self.records.insert(key, concurrent);
            }

            if let Some(kept) = self.records.get(subject.as_str()) {
                if &kept.subject != subject {
                    return Ok(GuardedReplace::WrongSubject);
                }
                if !replaceable_schema_versions.contains(&kept.schema_version) {
                    return Ok(GuardedReplace::SchemaNotReplaceable {
                        schema_version: kept.schema_version,
                    });
                }
            }
            if std::mem::take(&mut self.fail_next_write) {
                return Err("disk full");
            }
            self.records.insert(subject.as_str().to_owned(), record);
            Ok(GuardedReplace::Replaced)
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

    fn stored(subject: AvaiaSubject, state: &AvaiaState) -> StoredAvaiaState {
        StoredAvaiaState {
            subject,
            schema_version: state.schema_version,
            payload: serde_json::to_vec(state).expect("state encodes"),
        }
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

    #[test]
    fn what_is_kept_comes_back_for_the_same_subject() {
        let mut store = Memory::default();
        let her = subject("avaia:1");
        let state = walking_to("lake");
        persist(&mut store, &her, &state).expect("persist");
        assert_eq!(restore(&store, &her), Ok(Restored::State(state)));
    }

    #[test]
    fn another_subjects_record_is_never_read_or_replaced_as_hers() {
        let mut store = Memory::default();
        store.records.insert(
            "avaia:1".into(),
            stored(subject("avaia:2"), &walking_to("park")),
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
        let payload = br#"{
            "schema_version": 1,
            "intent": "Explore",
            "last_proposal": null,
            "pause_reason": "OwnerControl"
        }"#;
        let mut store = Memory::default();
        store.records.insert(
            "avaia:1".into(),
            StoredAvaiaState {
                subject: subject("avaia:1"),
                schema_version: 1,
                payload: payload.to_vec(),
            },
        );

        let Ok(Restored::State(state)) = restore(&store, &subject("avaia:1")) else {
            panic!("a v1 record restores");
        };
        assert!(state.is_current_schema());
        assert_eq!(state.intent, Some(AvaiaIntent::Explore));
    }

    #[test]
    fn a_future_schema_is_quarantined_without_decoding_its_payload() {
        let her = subject("avaia:1");
        let newer = StoredAvaiaState {
            subject: her.clone(),
            schema_version: AVAIA_STATE_SCHEMA_VERSION + 1,
            payload: b"future bytes that are not AvaiaState JSON".to_vec(),
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
        assert_eq!(restored.into_state(), AvaiaState::new());
        assert_eq!(
            persist(&mut store, &her, &AvaiaState::new()),
            Err(StateStoreError::WouldOverwriteUnsupported {
                schema_version: AVAIA_STATE_SCHEMA_VERSION + 1
            })
        );
        assert_eq!(store.records["avaia:1"], newer);
    }

    #[test]
    fn a_concurrent_future_write_cannot_be_overwritten() {
        let mut store = Memory::default();
        let her = subject("avaia:1");
        persist(&mut store, &her, &walking_to("lake")).expect("initial persist");

        let newer = StoredAvaiaState {
            subject: her.clone(),
            schema_version: AVAIA_STATE_SCHEMA_VERSION + 1,
            payload: b"future".to_vec(),
        };
        store.install_before_guard = Some(("avaia:1".into(), newer.clone()));

        assert_eq!(
            persist(&mut store, &her, &walking_to("park")),
            Err(StateStoreError::WouldOverwriteUnsupported {
                schema_version: AVAIA_STATE_SCHEMA_VERSION + 1
            })
        );
        assert_eq!(store.records["avaia:1"], newer);
    }

    #[test]
    fn a_known_schema_with_bad_or_mismatched_payload_is_rejected() {
        let her = subject("avaia:1");
        let mut store = Memory::default();
        store.records.insert(
            "avaia:1".into(),
            StoredAvaiaState {
                subject: her.clone(),
                schema_version: AVAIA_STATE_SCHEMA_VERSION,
                payload: b"not json".to_vec(),
            },
        );
        assert_eq!(
            restore(&store, &her),
            Err(StateStoreError::InvalidPayload {
                schema_version: AVAIA_STATE_SCHEMA_VERSION
            })
        );

        store.records.insert(
            "avaia:1".into(),
            StoredAvaiaState {
                subject: her.clone(),
                schema_version: 1,
                payload: serde_json::to_vec(&AvaiaState::new()).expect("encode"),
            },
        );
        assert_eq!(
            restore(&store, &her),
            Err(StateStoreError::InvalidPayload { schema_version: 1 })
        );
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
