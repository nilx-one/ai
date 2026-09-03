// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

#![forbid(unsafe_code)]

use aiai_runtime::ActivationTransition;

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
    /// Maps a product runtime-mode transition onto the generic foundation gate.
    #[must_use]
    pub const fn activation_transition(self) -> ActivationTransition {
        match self {
            Self::Spectate => ActivationTransition::Wake,
            Self::Manual => ActivationTransition::Quiesce,
            Self::Offline => ActivationTransition::Settle,
        }
    }
}

#[cfg(test)]
mod tests {
    use aiai_runtime::{ActivationState, ActivationTransition};

    use super::{
        AvaiaActionProposal, AvaiaControlMode, MapTargetId, NavigationProposalError,
    };

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
    fn control_modes_bind_to_foundation_activation_transitions() {
        assert_eq!(
            AvaiaControlMode::Spectate.activation_transition(),
            ActivationTransition::Wake
        );
        assert_eq!(
            AvaiaControlMode::Manual.activation_transition(),
            ActivationTransition::Quiesce
        );
        assert_eq!(
            AvaiaControlMode::Offline.activation_transition(),
            ActivationTransition::Settle
        );
    }

    #[test]
    fn product_modes_walk_the_foundation_activation_cycle() {
        let active = ActivationState::Dormant
            .apply(AvaiaControlMode::Spectate.activation_transition())
            .unwrap();
        assert_eq!(active, ActivationState::Active);

        let quiescing = active
            .apply(AvaiaControlMode::Manual.activation_transition())
            .unwrap();
        assert_eq!(quiescing, ActivationState::Quiescing);

        let dormant = quiescing
            .apply(AvaiaControlMode::Offline.activation_transition())
            .unwrap();
        assert_eq!(dormant, ActivationState::Dormant);
    }
}
