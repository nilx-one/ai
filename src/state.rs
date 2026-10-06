// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

//! Product-owned continuity state for Avaia.
//! This state survives replacement of the inference runtime. It describes what Avaia is
//! currently heading for and the last decision she made; it never records whether an
//! interaction happened. Protocol truth remains owned by 0x1/core.
//!
//! There is no pending step in it. A decision is not waiting on anyone: it becomes what she
//! is doing the moment she makes it ([`AvaiaState::decide`]). An owner's navigation position
//! becomes what she heads for next ([`AvaiaState::head_for`]), and once she arrives she is
//! back to her own choices ([`AvaiaState::arrive`]).

use crate::{AvaiaDecision, MapTargetId, MenuDecision, OwnerWaypoint};
use serde::{Deserialize, Serialize};

/// v2 renamed `last_proposal` to `last_decision` and the pause reasons that spoke of
/// control and admission. A v1 record reads field for field; see [`AvaiaState::upgraded`].
pub const AVAIA_STATE_SCHEMA_VERSION: u16 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AvaiaIntent {
    /// Nothing in particular: her own curiosity.
    Explore,
    /// A target she chose.
    NavigateTo(MapTargetId),
    /// A navigation position her owner set, which she is heading for.
    OwnerWaypoint(MapTargetId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AvaiaPauseReason {
    /// Her owner is at the wheel of their own Bond, so she rests.
    #[serde(alias = "OwnerControl")]
    OwnerAtWheel,
    /// Nothing was offered to choose between.
    #[serde(alias = "NoAdmissibleAction")]
    NothingToChoose,
    RuntimeUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AvaiaState {
    pub schema_version: u16,
    pub intent: Option<AvaiaIntent>,
    /// What she last decided. A decision is not an interaction or a completion record.
    #[serde(alias = "last_proposal")]
    pub last_decision: Option<AvaiaDecision>,
    pub pause_reason: Option<AvaiaPauseReason>,
}

impl Default for AvaiaState {
    fn default() -> Self {
        Self::new()
    }
}

impl AvaiaState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            schema_version: AVAIA_STATE_SCHEMA_VERSION,
            intent: None,
            last_decision: None,
            pause_reason: None,
        }
    }

    /// She decided. The decision is what she now does — walking to a target, or stopping —
    /// with nothing between the choice and the act.
    ///
    /// It takes a [`MenuDecision`], never a bare [`AvaiaDecision`]: what she carries out is
    /// always one of the choices a [`crate::DecisionMenu`] offered, which is where a walk is
    /// kept sound.
    pub fn decide(&mut self, decision: MenuDecision) {
        let decision = decision.into_decision();
        self.intent = match &decision {
            AvaiaDecision::NavigateTo { target } => Some(AvaiaIntent::NavigateTo(target.clone())),
            AvaiaDecision::StopNavigation => Some(AvaiaIntent::Explore),
        };
        self.last_decision = Some(decision);
        self.pause_reason = None;
    }

    /// Her owner set a navigation position: she heads there. It is not her decision, so it
    /// leaves `last_decision` as it was.
    pub fn head_for(&mut self, waypoint: OwnerWaypoint) {
        self.intent = Some(AvaiaIntent::OwnerWaypoint(waypoint.target));
        self.pause_reason = None;
    }

    /// She got where she was going, her owner's position or her own target, and carries on
    /// with her own curiosity from there.
    pub fn arrive(&mut self) {
        if matches!(
            self.intent,
            Some(AvaiaIntent::NavigateTo(_) | AvaiaIntent::OwnerWaypoint(_))
        ) {
            self.intent = Some(AvaiaIntent::Explore);
        }
    }

    /// Bounded product vocabulary supplied to an inference adapter.
    #[must_use]
    pub fn model_context(&self) -> AvaiaModelContext {
        AvaiaModelContext {
            intent: self.intent.clone(),
            last_decision: self.last_decision.clone(),
            pause_reason: self.pause_reason.clone(),
        }
    }

    #[must_use]
    pub const fn is_current_schema(&self) -> bool {
        self.schema_version == AVAIA_STATE_SCHEMA_VERSION
    }

    /// The same state in the current schema, when this crate can read it: a v1 record maps
    /// field for field. `None` for a schema newer than this crate, which a store keeps as it
    /// is rather than rewriting it.
    #[must_use]
    pub fn upgraded(self) -> Option<Self> {
        match self.schema_version {
            1 | AVAIA_STATE_SCHEMA_VERSION => Some(Self {
                schema_version: AVAIA_STATE_SCHEMA_VERSION,
                ..self
            }),
            _ => None,
        }
    }
}

/// One-way, model-facing projection of [`AvaiaState`]. Never deserialized: it is not a
/// storage format and must not be round-tripped back into product state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AvaiaModelContext {
    pub intent: Option<AvaiaIntent>,
    pub last_decision: Option<AvaiaDecision>,
    pub pause_reason: Option<AvaiaPauseReason>,
}

#[cfg(test)]
mod tests {
    use super::{AVAIA_STATE_SCHEMA_VERSION, AvaiaIntent, AvaiaPauseReason, AvaiaState};
    use crate::{
        AvaiaDecision, DecisionMenu, MapTargetId, MenuDecision, OwnerWaypoint, StopAction,
    };

    fn target(value: &str) -> MapTargetId {
        MapTargetId::new(value).expect("target")
    }

    /// A decision the way she makes one: chosen from a menu.
    fn chosen(decoded: &str) -> MenuDecision {
        let targets = [target("lake"), target("park")];
        DecisionMenu::new(&targets, StopAction::Offered)
            .expect("menu")
            .decide(decoded)
            .expect("an offered choice")
    }

    #[test]
    fn new_state_is_empty_and_versioned() {
        let state = AvaiaState::new();
        assert_eq!(state.schema_version, AVAIA_STATE_SCHEMA_VERSION);
        assert!(state.intent.is_none());
        assert!(state.last_decision.is_none());
        assert!(state.pause_reason.is_none());
        assert!(state.is_current_schema());
    }

    #[test]
    fn state_round_trips_without_runtime_state() {
        let state = AvaiaState {
            schema_version: AVAIA_STATE_SCHEMA_VERSION,
            intent: Some(AvaiaIntent::NavigateTo(target("target-7"))),
            last_decision: Some(AvaiaDecision::NavigateTo {
                target: target("target-7"),
            }),
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
            last_decision: Some(AvaiaDecision::StopNavigation),
            pause_reason: Some(AvaiaPauseReason::OwnerAtWheel),
        };
        let context = state.model_context();
        assert_eq!(context.intent, state.intent);
        assert_eq!(context.last_decision, state.last_decision);
        assert_eq!(context.pause_reason, state.pause_reason);
    }

    #[test]
    fn newer_schema_is_not_current() {
        let state = AvaiaState {
            schema_version: AVAIA_STATE_SCHEMA_VERSION + 1,
            ..AvaiaState::new()
        };
        assert!(!state.is_current_schema());
        assert_eq!(state.upgraded(), None);
    }

    /// The point of the type: a decision is what she does, with no step in between.
    #[test]
    fn a_decision_is_what_she_does_at_once() {
        let mut state = AvaiaState::new();
        state.pause_reason = Some(AvaiaPauseReason::NothingToChoose);

        state.decide(chosen("navigate lake"));
        assert_eq!(state.intent, Some(AvaiaIntent::NavigateTo(target("lake"))));
        assert_eq!(state.pause_reason, None);

        state.decide(chosen("stop"));
        assert_eq!(state.intent, Some(AvaiaIntent::Explore));
        assert_eq!(state.last_decision, Some(AvaiaDecision::StopNavigation));
    }

    #[test]
    fn an_owner_position_is_where_she_heads_then_she_is_her_own_again() {
        let mut state = AvaiaState::new();
        state.decide(chosen("navigate park"));

        state.head_for(OwnerWaypoint {
            target: target("square"),
        });
        assert_eq!(
            state.intent,
            Some(AvaiaIntent::OwnerWaypoint(target("square")))
        );
        // The owner's position is not her decision.
        assert_eq!(
            state.last_decision,
            Some(AvaiaDecision::NavigateTo {
                target: target("park")
            })
        );

        state.arrive();
        assert_eq!(state.intent, Some(AvaiaIntent::Explore));
    }

    #[test]
    fn a_v1_record_reads_and_upgrades_field_for_field() {
        let v1 = r#"{
            "schema_version": 1,
            "intent": "Explore",
            "last_proposal": { "NavigateTo": { "target": "museum" } },
            "pause_reason": "NoAdmissibleAction"
        }"#;
        let read: AvaiaState = serde_json::from_str(v1).expect("v1 reads");
        assert!(!read.is_current_schema());

        let upgraded = read.upgraded().expect("v1 upgrades");
        assert!(upgraded.is_current_schema());
        assert_eq!(
            upgraded.last_decision,
            Some(AvaiaDecision::NavigateTo {
                target: target("museum")
            })
        );
        assert_eq!(
            upgraded.pause_reason,
            Some(AvaiaPauseReason::NothingToChoose)
        );

        let control: AvaiaPauseReason =
            serde_json::from_str(r#""OwnerControl""#).expect("v1 reason reads");
        assert_eq!(control, AvaiaPauseReason::OwnerAtWheel);
    }
}
