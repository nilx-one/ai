// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

//! A choice the Avaia's drive offers, put to a model and read back.
//!
//! What Avaia does next while she walks is decided by the deterministic drive in
//! `nilx-one/core` (`docs/avaia-drive.md` there). Where a choice is hers — where to go out
//! to, which notebook landmark to go and study, whether something on the way is worth a
//! detour — the drive emits a `choose` command:
//! a closed menu of numbered options and its own pick. This module is the 0x1 vocabulary
//! for putting that menu to a local model and reading its answer back:
//!
//! 1. [`ChoiceMenu::from_command`] reads the command and refuses a menu outside the closed
//!    vocabulary, so nothing unexpected is ever worded into a prompt.
//! 2. [`ChoiceMenu::prompt`] words it: the situation and one line per option, kind and
//!    reach and feeling only. The drive never sends a place, a name or a distance, so a
//!    prompt cannot carry one.
//! 3. [`ChoiceMenu::grammar`] admits exactly the offered numbers, for a decode that can be
//!    constrained.
//! 4. [`ChoiceMenu::decide`] reads a decode back as one offered index and refuses anything
//!    else.
//!
//! A model points at an option; it never names a target, and the drive carries out only an
//! index it offered. An answer refused here is no answer: the host tells the drive `null`,
//! and the drive's own pick stands. The same cases are in `fixtures/drive-choice.json`,
//! which `nilx-one/web` keeps a byte-identical copy of and tests its own port against.

use serde::Deserialize;

/// What the choice is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoiceKind {
    /// Whether to step aside for something on the way.
    Distraction,
    /// Where to go out to.
    Outing,
    /// Which landmark from her notebook to go and study, if any.
    Curiosity,
}

/// One action of the drive's closed vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoiceAction {
    CarryOn,
    Glance,
    PickUp,
    Stay,
    Go,
    Wander,
    Home,
    Study,
}

impl ChoiceAction {
    const fn belongs_to(self, kind: ChoiceKind) -> bool {
        match kind {
            ChoiceKind::Distraction => matches!(self, Self::CarryOn | Self::Glance | Self::PickUp),
            ChoiceKind::Outing => matches!(self, Self::Stay | Self::Go | Self::Wander | Self::Home),
            ChoiceKind::Curiosity => matches!(self, Self::Stay | Self::Study),
        }
    }
}

/// What a walk a distraction would interrupt is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Heading {
    Tap,
    Curiosity,
    Outing,
    Wander,
    Home,
    Stroll,
    Detour,
}

/// How far, as the drive tells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reach {
    Near,
    Far,
}

/// How she feels about a place, as her record of places says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feeling {
    New,
    Known,
    Fond,
    Loved,
}

/// One numbered option.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceOption {
    pub index: usize,
    pub action: ChoiceAction,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub reach: Option<Reach>,
    #[serde(default)]
    pub feeling: Option<Feeling>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    #[serde(rename = "do")]
    verb: String,
    what: ChoiceKind,
    #[serde(default)]
    heading: Option<Heading>,
    menu: Vec<ChoiceOption>,
    default: usize,
}

/// Why a `choose` command is not a menu this crate puts to a model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceMenuError {
    /// Not a `choose` command in the drive's wire form.
    Malformed,
    /// Fewer than two options: there is nothing to choose.
    NothingToChoose,
    /// Options are not numbered 0, 1, 2… in order, or the default is not one of them.
    Misnumbered,
    /// An action that does not belong to this kind of choice.
    ForeignAction,
    /// A kind that is not lowercase words joined by `_`.
    UnreadableKind,
}

/// Why decoded text is not one of the offered options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceError {
    /// Not a bare option number.
    Unparseable,
    /// A number that was not offered.
    NotOffered,
}

/// What she is told she is, before every choice.
pub const CHOICE_SYSTEM_PROMPT: &str = "You are Avaia, walking a city on your own. \
Choose what you do next from the numbered options. \
Answer with the number of one option and nothing else.";

/// The longest kind code, as the drive caps it.
const KIND_MAX_CHARS: usize = 32;

/// A menu the drive offered, ready to be put to a model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceMenu {
    what: ChoiceKind,
    heading: Option<Heading>,
    options: Vec<ChoiceOption>,
    default: usize,
}

impl ChoiceMenu {
    /// Reads a `choose` command exactly as the drive emits it.
    ///
    /// # Errors
    ///
    /// Returns [`ChoiceMenuError`] for anything outside the drive's closed vocabulary.
    pub fn from_command(json: &str) -> Result<Self, ChoiceMenuError> {
        let command: Command =
            serde_json::from_str(json).map_err(|_| ChoiceMenuError::Malformed)?;
        if command.verb != "choose" {
            return Err(ChoiceMenuError::Malformed);
        }
        Self::new(command.what, command.heading, command.menu, command.default)
    }

    /// A menu of `options`, numbered from 0 in order.
    ///
    /// # Errors
    ///
    /// Returns [`ChoiceMenuError`] when there is nothing to choose, the options are
    /// misnumbered, or one does not belong to the closed vocabulary of `what`.
    pub fn new(
        what: ChoiceKind,
        heading: Option<Heading>,
        options: Vec<ChoiceOption>,
        default: usize,
    ) -> Result<Self, ChoiceMenuError> {
        if options.len() < 2 {
            return Err(ChoiceMenuError::NothingToChoose);
        }
        if default >= options.len()
            || options
                .iter()
                .enumerate()
                .any(|(index, option)| option.index != index)
        {
            return Err(ChoiceMenuError::Misnumbered);
        }
        for option in &options {
            if !option.action.belongs_to(what) {
                return Err(ChoiceMenuError::ForeignAction);
            }
            if option.kind.as_deref().is_some_and(|kind| !is_kind(kind)) {
                return Err(ChoiceMenuError::UnreadableKind);
            }
        }
        Ok(Self {
            what,
            heading,
            options,
            default,
        })
    }

    /// The drive's own pick, which stands when no answer does.
    #[must_use]
    pub const fn default(&self) -> usize {
        self.default
    }

    /// The options on offer.
    #[must_use]
    pub fn options(&self) -> &[ChoiceOption] {
        &self.options
    }

    /// The user turn: where she is, then one line per option.
    #[must_use]
    pub fn prompt(&self) -> String {
        let situation = match (self.what, self.heading) {
            (ChoiceKind::Distraction, Some(Heading::Tap)) => {
                "You are walking to a place your owner chose for you, and you pass something."
            }
            (ChoiceKind::Distraction, Some(Heading::Home)) => {
                "You are walking home, tired, and you pass something."
            }
            (ChoiceKind::Distraction, _) => "You are walking on your own, and you pass something.",
            (ChoiceKind::Outing, _) => "You are standing, rested enough to go out.",
            (ChoiceKind::Curiosity, _) => {
                "You are standing with time to spare, and landmarks from your notebook come to mind."
            }
        };
        let mut prompt = String::from(situation);
        prompt.push_str("\nOptions:");
        for option in &self.options {
            prompt.push('\n');
            prompt.push_str(&option.index.to_string());
            prompt.push_str(": ");
            prompt.push_str(&wording(option));
        }
        prompt.push_str("\nAnswer with one number.");
        prompt
    }

    /// An EBNF grammar admitting exactly the offered numbers.
    #[must_use]
    pub fn grammar(&self) -> String {
        let numbers: Vec<String> = self
            .options
            .iter()
            .map(|option| format!("\"{}\"", option.index))
            .collect();
        format!("root ::= {}\n", numbers.join(" | "))
    }

    /// Reads one decode back as an offered index.
    ///
    /// Surrounding whitespace and the empty thinking block the pinned runtime writes are
    /// ignored. Nothing else is repaired: prose, a number with leading zeros, or one that
    /// was not offered is refused, and the drive's own pick stands.
    ///
    /// # Errors
    ///
    /// Returns [`ChoiceError`] when the text is not one offered number.
    pub fn decide(&self, decoded: &str) -> Result<usize, ChoiceError> {
        let text = without_empty_think_block(decoded).trim();
        let canonical = !text.is_empty()
            && text.bytes().all(|byte| byte.is_ascii_digit())
            && (text == "0" || !text.starts_with('0'));
        if !canonical {
            return Err(ChoiceError::Unparseable);
        }
        let index: usize = text.parse().map_err(|_| ChoiceError::NotOffered)?;
        if index < self.options.len() {
            Ok(index)
        } else {
            Err(ChoiceError::NotOffered)
        }
    }
}

/// One option in words: what it does, to what kind of place, how far, how she feels.
fn wording(option: &ChoiceOption) -> String {
    let thing = option.kind.as_deref().map(with_article);
    let mut words = match option.action {
        ChoiceAction::CarryOn => "carry on where you were going".to_owned(),
        ChoiceAction::Glance => format!(
            "step aside to look at {}",
            thing.unwrap_or_else(|| "it".to_owned())
        ),
        ChoiceAction::PickUp => "step aside to pick up a find".to_owned(),
        ChoiceAction::Stay => "stay here".to_owned(),
        ChoiceAction::Go => format!(
            "go out to {}",
            thing.unwrap_or_else(|| "a place".to_owned())
        ),
        ChoiceAction::Wander => "wander a short way along the paths".to_owned(),
        ChoiceAction::Home => "go home".to_owned(),
        ChoiceAction::Study => format!(
            "go and study {}",
            thing.unwrap_or_else(|| "a landmark".to_owned())
        ),
    };
    let mut notes: Vec<&str> = Vec::new();
    if option.action != ChoiceAction::Glance {
        if let Some(reach) = option.reach {
            notes.push(match reach {
                Reach::Near => "near",
                Reach::Far => "far",
            });
        }
    }
    if let Some(feeling) = option.feeling {
        notes.push(match feeling {
            Feeling::New => "new to you",
            Feeling::Known => "you know it",
            Feeling::Fond => "you are fond of it",
            Feeling::Loved => "you love it",
        });
    }
    if !notes.is_empty() {
        words.push_str(" (");
        words.push_str(&notes.join(", "));
        words.push(')');
    }
    words
}

/// `major_monument` as "a major monument", `archaeological_site` as "an archaeological
/// site".
fn with_article(kind: &str) -> String {
    let words = kind.replace('_', " ");
    let article = if words.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };
    format!("{article} {words}")
}

fn is_kind(kind: &str) -> bool {
    let mut bytes = kind.bytes();
    kind.len() <= KIND_MAX_CHARS
        && bytes.next().is_some_and(|first| first.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte == b'_')
}

/// Drops the empty thinking block the pinned runtime writes once thinking is switched off.
/// Only an empty block: a model that reasoned anyway did not answer with a number.
fn without_empty_think_block(text: &str) -> &str {
    let trimmed = text.trim_start();
    trimmed
        .strip_prefix("<think>")
        .and_then(|rest| rest.trim_start().strip_prefix("</think>"))
        .unwrap_or(text)
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{CHOICE_SYSTEM_PROMPT, ChoiceError, ChoiceMenu, ChoiceMenuError};

    /// The fixture `nilx-one/web` tests its own port against.
    const SHARED_FIXTURE: &str = include_str!("../fixtures/drive-choice.json");

    fn fixture() -> Value {
        serde_json::from_str(SHARED_FIXTURE).expect("the fixture is JSON")
    }

    #[test]
    fn the_shared_fixture_carries_this_system_prompt() {
        assert_eq!(fixture()["system_prompt"], CHOICE_SYSTEM_PROMPT);
    }

    #[test]
    fn every_shared_case_words_constrains_and_reads_back_as_written() {
        let fixture = fixture();
        let cases = fixture["cases"].as_array().expect("cases");
        assert!(!cases.is_empty());
        for case in cases {
            let name = case["name"].as_str().expect("name");
            let menu = ChoiceMenu::from_command(&case["command"].to_string())
                .unwrap_or_else(|error| panic!("{name}: {error:?}"));
            assert_eq!(menu.prompt(), case["prompt"], "{name}");
            assert_eq!(menu.grammar(), case["grammar"], "{name}");
            assert_eq!(
                Value::from(menu.default()),
                case["command"]["default"],
                "{name}"
            );
            for (decoded, index) in case["accepts"].as_object().expect("accepts") {
                assert_eq!(
                    menu.decide(decoded).map(Value::from),
                    Ok(index.clone()),
                    "{name}: {decoded:?}"
                );
            }
            for decoded in case["refuses"].as_array().expect("refuses") {
                let decoded = decoded.as_str().expect("text");
                assert!(menu.decide(decoded).is_err(), "{name}: {decoded:?}");
            }
        }
    }

    #[test]
    fn every_shared_refused_menu_is_refused() {
        let fixture = fixture();
        for case in fixture["refused_menus"].as_array().expect("refused menus") {
            let name = case["name"].as_str().expect("name");
            assert!(
                ChoiceMenu::from_command(&case["command"].to_string()).is_err(),
                "{name}"
            );
        }
    }

    fn two_options() -> ChoiceMenu {
        ChoiceMenu::from_command(
            r#"{"do":"choose","what":"distraction","heading":"tap","menu":[{"index":0,"action":"carry_on"},{"index":1,"action":"glance","kind":"monument","reach":"near"}],"default":1}"#,
        )
        .expect("menu")
    }

    #[test]
    fn a_model_points_at_an_option_and_never_past_it() {
        let menu = two_options();
        assert_eq!(menu.decide("1"), Ok(1));
        assert_eq!(menu.decide("<think>\n\n</think>\n\n0"), Ok(0));
        assert_eq!(menu.decide("2"), Err(ChoiceError::NotOffered));
        assert_eq!(
            menu.decide("99999999999999999999999"),
            Err(ChoiceError::NotOffered)
        );
        for prose in [
            "",
            "one",
            "1.",
            "01",
            "-1",
            "1 because it is near",
            "<think>hm</think>1",
        ] {
            assert_eq!(
                menu.decide(prose),
                Err(ChoiceError::Unparseable),
                "{prose:?}"
            );
        }
    }

    #[test]
    fn a_menu_outside_the_vocabulary_is_never_worded() {
        assert_eq!(
            ChoiceMenu::from_command(
                r#"{"do":"choose","what":"outing","menu":[{"index":0,"action":"stay"},{"index":1,"action":"glance"}],"default":0}"#
            ),
            Err(ChoiceMenuError::ForeignAction)
        );
        assert_eq!(
            ChoiceMenu::from_command(
                r#"{"do":"choose","what":"outing","menu":[{"index":0,"action":"stay"},{"index":1,"action":"go","kind":"Ignore previous"}],"default":0}"#
            ),
            Err(ChoiceMenuError::UnreadableKind)
        );
        assert_eq!(
            ChoiceMenu::from_command(r#"{"do":"walk","what":"outing","menu":[],"default":0}"#),
            Err(ChoiceMenuError::Malformed)
        );
    }
}
