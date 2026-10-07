// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

#![forbid(unsafe_code)]

mod choice;
mod decision;
mod inference;
mod routing;
mod state;
mod store;

use aiai_runtime::ActivationState;
use serde::{Deserialize, Serialize};

pub use choice::{
    CHOICE_SYSTEM_PROMPT, ChoiceAction, ChoiceError, ChoiceKind, ChoiceMenu, ChoiceMenuError,
    ChoiceOption, Feeling, Heading, Reach,
};
pub use decision::{DecisionError, DecisionMenu, DecisionMenuError, MenuDecision, StopAction};
pub use inference::{
    Admission, DeviceCapability, DeviceLimit, Ineligible, Licence, LocalModel, MemoryBudget,
    ModelFamily, default_local_model, eligible_local_models, find_local_model, runtime_floor,
    select_local_model, served_models,
};
pub use routing::{AvaiaFailureReport, FailureSink, RoutedFailure, route_failure};
pub use state::{
    AVAIA_STATE_SCHEMA_VERSION, AvaiaIntent, AvaiaModelContext, AvaiaPauseReason, AvaiaState,
};
pub use store::{
    AvaiaStateStore, AvaiaSubject, EmptySubject, Restored, StateStoreError, StoredAvaiaState,
    persist, restore,
};

/// A navigation position an owner set: the only way an owner influences where Avaia goes.
///
/// It is not a command she waits for and not a permission she needs. It is a place an owner
/// puts on her map: she heads there, and once there carries on with her own decisions from
/// where she stands. Nothing else an owner does steers her walk.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OwnerWaypoint {
    pub target: MapTargetId,
}

/// Opaque map target selected and owned by the product world layer.
///
/// Avaia may refer to a target but does not mint or reinterpret its coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MapTargetId(String);

impl MapTargetId {
    /// Creates a non-empty target identifier.
    ///
    /// # Errors
    /// Returns [`NavigationError::EmptyTargetId`] for an empty identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, NavigationError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(NavigationError::EmptyTargetId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for MapTargetId {
    type Error = NavigationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<MapTargetId> for String {
    fn from(value: MapTargetId) -> Self {
        value.0
    }
}

/// What Avaia decided to do next, as a record.
///
/// Where she goes is hers. A decision is carried out as it was made: nothing admits it,
/// approves it, or grants a scope for it, step by step or once per walk. Moving her body is
/// not a world mutation — it asserts no presence, attendance, interaction or `BondChain` fact
/// — so the 0x1 authority boundary has nothing to rule on here. What keeps a decision
/// sound is upstream of it: she chooses only among targets the world layer resolved
/// ([`DecisionMenu`]), on ground she may walk on, and only while she is at the wheel
/// ([`AvaiaControlMode::Spectate`]). An owner influences her only through an
/// [`OwnerWaypoint`].
///
/// This type is what a decision says, for state and for a model's context. Anyone can write
/// one, so it moves nothing: [`AvaiaState::decide`] carries out only a [`MenuDecision`],
/// which only a [`DecisionMenu`] makes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvaiaDecision {
    NavigateTo { target: MapTargetId },
    StopNavigation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationError {
    EmptyTargetId,
}

impl std::fmt::Display for NavigationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTargetId => write!(f, "map target id must not be empty"),
        }
    }
}

impl std::error::Error for NavigationError {}

/// Who is at the wheel: 0x1 owner/AI runtime modes.
///
/// `Spectate` is Avaia at the wheel, living her own walk while her owner watches. `Manual`
/// is the owner at the wheel of their own Bond, and Avaia rests. A mode says when she lives,
/// not what she may do while she does. These remain product semantics; the shared
/// foundation only supplies the generic activation state machine that enforces when
/// computation may run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvaiaControlMode {
    Spectate,
    Manual,
    Offline,
}

impl AvaiaControlMode {
    /// Maps a product runtime mode onto the activation state it names.
    ///
    /// A mode names a state, not an edge. Mapping one onto a transition only works from
    /// the single state that edge leaves, so the target is the state and the foundation's
    /// `ensure_activation` resolves the step — which makes re-applying the mode a session
    /// is already in a no-op rather than an undefined transition. A client renders the
    /// current mode on every reconnect, so that case is the common one.
    #[must_use]
    pub const fn activation_state(self) -> ActivationState {
        match self {
            Self::Spectate => ActivationState::Active,
            Self::Manual => ActivationState::Quiescing,
            Self::Offline => ActivationState::Dormant,
        }
    }
}

#[cfg(test)]
mod tests {
    use aiai_runtime::ActivationState;

    use super::{AvaiaControlMode, AvaiaDecision, MapTargetId, NavigationError};

    #[test]
    fn empty_target_is_rejected() {
        assert_eq!(MapTargetId::new("   "), Err(NavigationError::EmptyTargetId));
    }

    #[test]
    fn deserializing_an_empty_target_id_is_rejected() {
        let result: Result<MapTargetId, _> = serde_json::from_str("\"\"");
        assert!(result.is_err());
    }

    #[test]
    fn map_target_id_round_trips_through_json() {
        let target = MapTargetId::new("map-target-42").unwrap();
        let bytes = serde_json::to_vec(&target).expect("serialize");
        let restored: MapTargetId = serde_json::from_slice(&bytes).expect("deserialize");
        assert_eq!(restored, target);
    }

    #[test]
    fn stopping_navigation_needs_no_spatial_payload() {
        assert_eq!(AvaiaDecision::StopNavigation, AvaiaDecision::StopNavigation);
    }

    #[test]
    fn control_modes_bind_to_foundation_activation_states() {
        assert_eq!(
            AvaiaControlMode::Spectate.activation_state(),
            ActivationState::Active
        );
        assert_eq!(
            AvaiaControlMode::Manual.activation_state(),
            ActivationState::Quiescing
        );
        assert_eq!(
            AvaiaControlMode::Offline.activation_state(),
            ActivationState::Dormant
        );
    }

    #[test]
    fn product_modes_walk_the_foundation_activation_cycle() {
        let mut state = ActivationState::Dormant;

        for mode in [
            AvaiaControlMode::Spectate,
            AvaiaControlMode::Manual,
            AvaiaControlMode::Offline,
        ] {
            if let Some(transition) = state.transition_to(mode.activation_state()).unwrap() {
                state = state.apply(transition).unwrap();
            }
            assert_eq!(state, mode.activation_state());
        }
    }

    /// A client re-sends the mode it is already in on every reconnect. That must resolve
    /// to no step at all rather than to a transition the state machine does not define.
    #[test]
    fn re_applying_the_current_mode_resolves_to_no_step() {
        assert_eq!(
            ActivationState::Active.transition_to(AvaiaControlMode::Spectate.activation_state()),
            Ok(None)
        );
    }

    /// Leaving MANUAL for SPECTATE is two explicit steps: settling is the owner's
    /// assertion that in-flight work reached its boundary, not one this crate makes.
    #[test]
    fn quiescing_does_not_resolve_back_into_activity() {
        assert!(
            ActivationState::Quiescing
                .transition_to(AvaiaControlMode::Spectate.activation_state())
                .is_err()
        );
    }
}
