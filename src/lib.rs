// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

#![forbid(unsafe_code)]

mod routing;

use aiai_runtime::ActivationState;

pub use routing::{AvaiaFailureReport, FailureSink, RoutedFailure, route_failure};

/// Opaque map target selected and owned by the product world layer.
///
/// Avaia may refer to a target but does not mint or reinterpret its coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapTargetId(String);

impl MapTargetId {
    /// Creates a non-empty target identifier.
    ///
    /// # Errors
    /// Returns [`NavigationProposalError::EmptyTargetId`] for an empty identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, NavigationProposalError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(NavigationProposalError::EmptyTargetId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Product-specific action Avaia may propose.
///
/// This value is computation, not authority. Consumers must route it through the
/// 0x1 authority boundary before any world mutation is attempted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvaiaActionProposal {
    NavigateTo { target: MapTargetId },
    StopNavigation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationProposalError {
    EmptyTargetId,
}

impl AvaiaActionProposal {
    /// Builds a navigation proposal that references an already resolved map target.
    ///
    /// # Errors
    /// Returns [`NavigationProposalError::EmptyTargetId`] when `target_id` is empty.
    pub fn navigate_to(target_id: impl Into<String>) -> Result<Self, NavigationProposalError> {
        Ok(Self::NavigateTo {
            target: MapTargetId::new(target_id)?,
        })
    }
}

/// 0x1 owner/AI runtime modes.
///
/// These remain product semantics; the shared foundation only supplies the generic
/// activation state machine that enforces when computation may run.
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

    use super::{AvaiaActionProposal, AvaiaControlMode, MapTargetId, NavigationProposalError};

    #[test]
    fn navigate_to_keeps_target_opaque() {
        let proposal = AvaiaActionProposal::navigate_to("map-target-42").unwrap();

        assert_eq!(
            proposal,
            AvaiaActionProposal::NavigateTo {
                target: MapTargetId::new("map-target-42").unwrap(),
            }
        );
    }

    #[test]
    fn empty_target_is_rejected() {
        assert_eq!(
            AvaiaActionProposal::navigate_to("   "),
            Err(NavigationProposalError::EmptyTargetId)
        );
    }

    #[test]
    fn stopping_navigation_needs_no_spatial_payload() {
        assert_eq!(
            AvaiaActionProposal::StopNavigation,
            AvaiaActionProposal::StopNavigation
        );
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
