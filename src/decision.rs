// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

//! The closed set of actions offered for one decision.
//!
//! A model that chooses where to go is only choosing if what it produces cannot be anything
//! other than one of the choices. This module states that set twice, on purpose:
//!
//! 1. As a grammar. The pinned `WebLLM` accepts `response_format` with an EBNF grammar, and
//!    a decode constrained by [`DecisionMenu::grammar`] cannot emit a target the world layer
//!    did not resolve — the invalid choice is unrepresentable rather than merely rejected.
//! 2. As a parse. [`DecisionMenu::decide`] matches decoded text against the same set and
//!    refuses anything else.
//!
//! The second is not redundant. The foundation's adapter passes no `response_format` today,
//! so the grammar has nowhere to be applied yet, and this crate must hold the boundary
//! either way. It also holds if a future decode is unconstrained, partial, or produced by a
//! provider that ignores grammars.
//!
//! What leaves is an [`AvaiaActionProposal`] built from the caller's own [`MapTargetId`] —
//! never from text a model produced. A model may point at a target; it may not mint one. And
//! a proposal is still computation: the authority boundary decides whether anything is
//! attempted.

use crate::{AvaiaActionProposal, MapTargetId};

/// Whether stopping is one of the choices on this turn.
///
/// Stopping is only meaningful while something is under way, so it is offered by the caller
/// rather than assumed, and a decode that says `stop` when it was withheld is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopAction {
    /// `StopNavigation` is one of the choices.
    Offered,
    /// It is not.
    Withheld,
}

/// Why a menu could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionMenuError {
    /// No targets and no stop: there would be nothing to choose.
    NothingOffered,
    /// The same target was offered twice, which no world layer means to do.
    DuplicateTarget,
    /// A target identifier cannot be written as a grammar literal, so a constrained decode
    /// could not name it. Control characters are the case this catches.
    TargetNotExpressible,
}

/// Why decoded text is not a decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionError {
    /// The text is not one of the shapes the menu admits.
    Unparseable,
    /// A navigation target that was not offered. The model does not mint targets.
    TargetNotOffered,
    /// Stopping was chosen but not offered.
    StopNotOffered,
}

const NAVIGATE: &str = "navigate ";
const STOP: &str = "stop";

/// The actions Avaia may choose between on one turn.
///
/// Borrowed rather than owned: the targets belong to the world layer that resolved them, and
/// a menu is a view of that resolution for exactly as long as the decision takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecisionMenu<'a> {
    targets: &'a [MapTargetId],
    stop: StopAction,
}

impl<'a> DecisionMenu<'a> {
    /// Offers `targets`, and stopping when `stop` says so.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionMenuError`] when there is nothing to choose between, when a target
    /// repeats, or when one cannot be written into a grammar.
    pub fn new(targets: &'a [MapTargetId], stop: StopAction) -> Result<Self, DecisionMenuError> {
        if targets.is_empty() && stop == StopAction::Withheld {
            return Err(DecisionMenuError::NothingOffered);
        }
        for (index, target) in targets.iter().enumerate() {
            if !is_expressible(target.as_str()) {
                return Err(DecisionMenuError::TargetNotExpressible);
            }
            if targets[..index].contains(target) {
                return Err(DecisionMenuError::DuplicateTarget);
            }
        }
        Ok(Self { targets, stop })
    }

    /// Returns the targets on offer.
    #[must_use]
    pub const fn targets(&self) -> &'a [MapTargetId] {
        self.targets
    }

    /// Returns whether stopping is on offer.
    #[must_use]
    pub const fn stop(&self) -> StopAction {
        self.stop
    }

    /// Returns an EBNF grammar admitting exactly this menu and nothing else.
    ///
    /// Suitable for the pinned `WebLLM`'s `response_format: { type: "grammar" }`. The rule
    /// named `root` is the entry point; every target appears as a literal, so a constrained
    /// decode selects between whole identifiers rather than assembling one.
    #[must_use]
    pub fn grammar(&self) -> String {
        let mut alternatives: Vec<String> = Vec::new();
        if !self.targets.is_empty() {
            alternatives.push(format!("{} target", literal(NAVIGATE)));
        }
        if self.stop == StopAction::Offered {
            alternatives.push(literal(STOP));
        }

        let mut grammar = format!("root ::= {}\n", alternatives.join(" | "));
        if !self.targets.is_empty() {
            let targets: Vec<String> = self
                .targets
                .iter()
                .map(|target| literal(target.as_str()))
                .collect();
            grammar.push_str("target ::= ");
            grammar.push_str(&targets.join(" | "));
            grammar.push('\n');
        }
        grammar
    }

    /// Reads one decoded decision back as a proposal over an offered target.
    ///
    /// Surrounding whitespace is ignored, because a decode routinely carries a trailing
    /// newline. Nothing else is repaired: a decision this menu does not contain is refused
    /// rather than guessed at, and the returned target is the caller's own value.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] when the text is not one of this menu's choices.
    pub fn decide(&self, decoded: &str) -> Result<AvaiaActionProposal, DecisionError> {
        let decoded = decoded.trim();

        if decoded == STOP {
            return match self.stop {
                StopAction::Offered => Ok(AvaiaActionProposal::StopNavigation),
                StopAction::Withheld => Err(DecisionError::StopNotOffered),
            };
        }

        let Some(named) = decoded.strip_prefix(NAVIGATE) else {
            return Err(DecisionError::Unparseable);
        };
        self.targets
            .iter()
            .find(|target| target.as_str() == named)
            .map(|target| AvaiaActionProposal::NavigateTo {
                target: target.clone(),
            })
            .ok_or(DecisionError::TargetNotOffered)
    }
}

/// Whether a value can be written as a grammar literal at all.
///
/// A grammar literal is a quoted string, so a quote or a backslash is escaped and survives.
/// A control character does not: it cannot be written literally and escaping it would put a
/// different value in the grammar than the world layer resolved.
fn is_expressible(value: &str) -> bool {
    !value.chars().any(char::is_control)
}

fn literal(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        if character == '"' || character == '\\' {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests {
    use super::{DecisionError, DecisionMenu, DecisionMenuError, StopAction};
    use crate::{AvaiaActionProposal, MapTargetId};

    fn target(value: &str) -> MapTargetId {
        MapTargetId::new(value).expect("non-empty target")
    }

    fn two_targets() -> Vec<MapTargetId> {
        vec![target("map-target-1"), target("map-target-2")]
    }

    #[test]
    fn the_grammar_admits_every_offered_target_and_stopping() {
        let targets = two_targets();
        let menu = DecisionMenu::new(&targets, StopAction::Offered).expect("menu");

        assert_eq!(
            menu.grammar(),
            "root ::= \"navigate \" target | \"stop\"\n\
             target ::= \"map-target-1\" | \"map-target-2\"\n"
        );
    }

    #[test]
    fn a_withheld_stop_is_not_in_the_grammar() {
        let targets = two_targets();
        let menu = DecisionMenu::new(&targets, StopAction::Withheld).expect("menu");

        assert!(!menu.grammar().contains("\"stop\""));
    }

    #[test]
    fn a_menu_of_only_stopping_names_no_target_rule() {
        let menu = DecisionMenu::new(&[], StopAction::Offered).expect("menu");

        assert_eq!(menu.grammar(), "root ::= \"stop\"\n");
    }

    #[test]
    fn a_menu_with_nothing_to_choose_is_refused() {
        assert_eq!(
            DecisionMenu::new(&[], StopAction::Withheld),
            Err(DecisionMenuError::NothingOffered)
        );
    }

    #[test]
    fn a_repeated_target_is_refused_rather_than_collapsed() {
        let targets = vec![target("map-target-1"), target("map-target-1")];

        assert_eq!(
            DecisionMenu::new(&targets, StopAction::Offered),
            Err(DecisionMenuError::DuplicateTarget)
        );
    }

    /// A target that cannot be written into the grammar would be offered by the parser and
    /// unreachable by the decode, which is worse than refusing the menu.
    #[test]
    fn a_target_a_grammar_cannot_name_is_refused() {
        let targets = vec![target("map\ntarget")];

        assert_eq!(
            DecisionMenu::new(&targets, StopAction::Offered),
            Err(DecisionMenuError::TargetNotExpressible)
        );
    }

    #[test]
    fn a_quote_in_a_target_is_escaped_rather_than_dropped() {
        let targets = vec![target("map\"target\\1")];
        let menu = DecisionMenu::new(&targets, StopAction::Withheld).expect("menu");

        assert!(menu.grammar().contains("\"map\\\"target\\\\1\""));
    }

    #[test]
    fn an_offered_target_decodes_to_a_proposal_over_the_callers_value() {
        let targets = two_targets();
        let menu = DecisionMenu::new(&targets, StopAction::Offered).expect("menu");

        assert_eq!(
            menu.decide("navigate map-target-2\n"),
            Ok(AvaiaActionProposal::NavigateTo {
                target: target("map-target-2")
            })
        );
    }

    /// The refusal this module exists for: a model may point at a target, never mint one.
    #[test]
    fn a_target_that_was_not_offered_is_refused() {
        let targets = two_targets();
        let menu = DecisionMenu::new(&targets, StopAction::Offered).expect("menu");

        assert_eq!(
            menu.decide("navigate map-target-9"),
            Err(DecisionError::TargetNotOffered)
        );
    }

    #[test]
    fn stopping_is_refused_when_it_was_not_offered() {
        let targets = two_targets();
        let menu = DecisionMenu::new(&targets, StopAction::Withheld).expect("menu");

        assert_eq!(menu.decide("stop"), Err(DecisionError::StopNotOffered));
        assert_eq!(
            DecisionMenu::new(&targets, StopAction::Offered)
                .expect("menu")
                .decide("  stop  "),
            Ok(AvaiaActionProposal::StopNavigation)
        );
    }

    #[test]
    fn prose_is_not_a_decision() {
        let targets = two_targets();
        let menu = DecisionMenu::new(&targets, StopAction::Offered).expect("menu");

        for decoded in [
            "",
            "I think I will go to map-target-1",
            "navigate",
            "navigatemap-target-1",
            "NAVIGATE map-target-1",
            "stop navigation",
        ] {
            assert_eq!(menu.decide(decoded), Err(DecisionError::Unparseable));
        }
    }

    /// Whitespace inside an identifier is part of it, so a target that contains a space is
    /// matched whole rather than split on.
    #[test]
    fn a_target_containing_a_space_is_matched_whole() {
        let targets = vec![target("old town square")];
        let menu = DecisionMenu::new(&targets, StopAction::Withheld).expect("menu");

        assert_eq!(
            menu.decide("navigate old town square"),
            Ok(AvaiaActionProposal::NavigateTo {
                target: target("old town square")
            })
        );
    }
}
