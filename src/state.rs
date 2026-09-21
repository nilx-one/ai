// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

//! Product-owned continuity state for Avaia.
//! This state survives replacement of the inference runtime. It describes what Avaia is
//! currently trying to do and the last proposal it produced; it never records whether an
//! interaction happened. Protocol truth remains owned by 0x1/core.

use crate::{AvaiaActionProposal, MapTargetId};
use serde::{Deserialize, Serialize};

pub const AVAIA_STATE_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvaiaIntent {
    Explore,
    NavigateTo(MapTargetId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvaiaPauseReason {
    OwnerControl,
    NoAdmissibleAction,
    RuntimeUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvaiaState {
    pub schema_version: u16,
    pub intent: Option<AvaiaIntent>,
    /// A proposal is not an admitted action or completion record.
    pub last_proposal: Option<AvaiaActionProposal>,
    pub pause_reason: Option<AvaiaPauseReason>,
}

impl Default for AvaiaState {
    fn default() -> Self { Self::new() }
}

impl AvaiaState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            schema_version: AVAIA_STATE_SCHEMA_VERSION,
            intent: None,
            last_proposal: None,
            pause_reason: None,
        }
    }

    /// Bounded product vocabulary supplied to an inference adapter.
    #[must_use]
    pub fn model_context(&self) -> AvaiaModelContext {
        AvaiaModelContext {
            intent: self.intent.clone(),
            last_proposal: self.last_proposal.clone(),
            pause_reason: self.pause_reason.clone(),
        }
    }

    #[must_use]
    pub const fn is_current_schema(&self) -> bool {
        self.schema_version == AVAIA_STATE_SCHEMA_VERSION
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvaiaModelContext {
    pub intent: Option<AvaiaIntent>,
    pub last_proposal: Option<AvaiaActionProposal>,
    pub pause_reason: Option<AvaiaPauseReason>,
}

#[cfg(test)]
mod tests {
    use super::{AvaiaIntent, AvaiaPauseReason, AvaiaState, AVAIA_STATE_SCHEMA_VERSION};
    use crate::{AvaiaActionProposal, MapTargetId};

    fn target(value: &str) -> MapTargetId {
        MapTargetId::new(value).expect("target")
    }

    #[test]
    fn new_state_is_empty_and_versioned() {
        let state = AvaiaState::new();
        assert_eq!(state.schema_version, AVAIA_STATE_SCHEMA_VERSION);
        assert!(state.intent.is_none());
        assert!(state.last_proposal.is_none());
        assert!(state.pause_reason.is_none());
        assert!(state.is_current_schema());
    }

    #[test]
    fn state_round_trips_without_runtime_state() {
        let state = AvaiaState {
            schema_version: AVAIA_STATE_SCHEMA_VERSION,
            intent: Some(AvaiaIntent::NavigateTo(target("target-7"))),
            last_proposal: Some(AvaiaActionProposal::NavigateTo { target: target("target-7") }),
            pause_reason: Some(AvaiaPauseReason::RuntimeUnavailable),
        };
        let bytes = serde_json::to_vec(&state).expect("serialize");
        let restored: AvaiaState = serde_json::from_slice(&bytes).expect("deserialize");
        assert_eq!(restored, state);
    }

    #[test]
    fn model_context_is_exactly_the_bounded_product_projection() {
        let state = AvaiaState {
            schema_version: AVAIA_STATE_SCHEMA_VERSION,
            intent: Some(AvaiaIntent::Explore),
            last_proposal: Some(AvaiaActionProposal::StopNavigation),
            pause_reason: Some(AvaiaPauseReason::OwnerControl),
        };
        let context = state.model_context();
        assert_eq!(context.intent, state.intent);
        assert_eq!(context.last_proposal, state.last_proposal);
        assert_eq!(context.pause_reason, state.pause_reason);
    }

    #[test]
    fn newer_schema_is_not_current() {
        let state = AvaiaState {
            schema_version: AVAIA_STATE_SCHEMA_VERSION + 1,
            ..AvaiaState::new()
        };
        assert!(!state.is_current_schema());
    }
}
