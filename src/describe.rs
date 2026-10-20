// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

//! How the Avaia describes a landmark she has stepped aside for, and what of a model's
//! answer may be shown.
//!
//! The drive in `nilx-one/core` decides *when* a landmark is described (its `describe`
//! command); it never sees the place. The host answers it here:
//!
//! 1. [`LandmarkFacts::from_json`] reads what the archive declares about the landmark,
//!    keeps only the closed set of facts in [`DESCRIBED_FACTS`], and drops any value that
//!    is long, multi-line or looks like markup. Nothing else about the place, and nothing
//!    about the person, ever reaches a prompt.
//! 2. [`LandmarkFacts::template`] is tier 0: one deterministic description in the host's
//!    language. It is the product, not a fallback: without a model, this is what she says
//!    and what the notebook keeps.
//! 3. [`DESCRIBE_SYSTEM_PROMPT`] and the template as the user turn ask a local model to say
//!    the same thing in her own words.
//! 4. [`admit`] decides whether that answer may be shown. A rephrasing that could carry a
//!    new fact (a number or a capitalised word the template does not have, a missing name,
//!    too long, markup, an unfinished sentence) is refused, and the template stands. A
//!    capitalised word passes when its first four letters are a template word's, so a
//!    declined name does, and an invented person does not.
//!
//! The same cases are in `fixtures/landmark-description.json`, which `nilx-one/web` keeps a
//! byte-identical copy of and tests its own port against.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The archive facts a description may use, in the order it uses them.
pub const DESCRIBED_FACTS: [&str; 5] = [
    "start_date",
    "artist_name",
    "architect",
    "height",
    "material",
];

/// What she is told before she retells a description.
pub const DESCRIBE_SYSTEM_PROMPT: &str = "You are Avaia, looking at a landmark on a walk. \
Retell the description you are given in your own words, in the same language, in at most two \
short sentences. Use only what it says: never add a date, a number, a name, a person, an event \
or a place it does not contain. Answer with the description and nothing else.";

/// The longest description that may be shown or kept, in characters.
pub const DESCRIPTION_MAX_CHARS: usize = 240;

/// Version of tier-0 wording, also carried through Core to the notebook.
pub const DESCRIPTION_TEMPLATE_VERSION: u16 = 1;

/// What the AI faculty returns to Core. Core never accepts a bare "done"
/// acknowledgement: it must receive the actual generated text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LandmarkDescriptionResult {
    pub text: String,
    pub produced_by: String,
    pub template_version: u16,
}

/// Executes Core's typed request with an optional local-model candidate.
///
/// The host transports the request and candidate, but cannot supply a
/// free-form prompt or decide which answer should be admitted. A malformed
/// request is an error. An unavailable or inadmissible model answer returns
/// tier 0, never invented content.
///
/// # Errors
///
/// Returns [`DescribeInputError`] when Core's archive-fact input is malformed.
pub fn execute_description(
    request_json: &str,
    model_candidate: Option<(&str, &str)>,
) -> Result<LandmarkDescriptionResult, DescribeInputError> {
    let facts = LandmarkFacts::from_json(request_json)?;
    let template = facts.template();
    let model_text = model_candidate.and_then(|(text, model_id)| {
        let valid_id = !model_id.is_empty()
            && model_id.len() <= 80
            && model_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':'));
        if valid_id {
            admit(&facts, text)
                .ok()
                .map(|accepted| (accepted, model_id))
        } else {
            None
        }
    });
    let (text, produced_by) = model_text.map_or_else(
        || (template, "template".to_owned()),
        |(text, id)| (text, id.to_owned()),
    );
    Ok(LandmarkDescriptionResult {
        text,
        produced_by,
        template_version: DESCRIPTION_TEMPLATE_VERSION,
    })
}

/// The longest name or fact value a description takes, in characters.
const VALUE_MAX_CHARS: usize = 80;

/// The longest kind code, as the drive caps it.
const KIND_MAX_CHARS: usize = 32;

/// The most sentences a description may have.
const MAX_SENTENCES: usize = 3;

/// The capitalised words a retelling may open a sentence with that are no name.
const OPENERS: [&str; 18] = [
    "It",
    "Its",
    "This",
    "The",
    "A",
    "An",
    "Here",
    "She",
    "Це",
    "Цей",
    "Ця",
    "Вона",
    "Тут",
    "Это",
    "Этот",
    "Эта",
    "Она",
    "Здесь",
];

/// How many leading letters two words must share to be the same word declined.
const STEM_CHARS: usize = 4;

/// The languages a description is worded in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DescribeLocale {
    Uk,
    En,
    Ru,
}

/// Why a landmark could not be described at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescribeInputError {
    /// Not the expected JSON shape.
    Malformed,
    /// A kind outside the drive's code alphabet.
    UnreadableKind,
    /// A kind label that is empty, long, multi-line or markup.
    UnreadableKindLabel,
}

/// Why a model's answer may not be shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescribeRefusal {
    /// Nothing, or longer than [`DESCRIPTION_MAX_CHARS`].
    Length,
    /// Markup, a control character or a line break.
    Markup,
    /// Not finished as a sentence, or more than three.
    Form,
    /// A number the template does not carry.
    NewNumber,
    /// The landmark's name is missing from it.
    NameMissing,
    /// A capitalised word, a name most likely, that the template does not have.
    NewName,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    locale: DescribeLocale,
    kind: String,
    kind_label: String,
    name: Option<String>,
    #[serde(default)]
    facts: serde_json::Map<String, Value>,
}

/// What the archive declares about one landmark, as far as a description may use it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandmarkFacts {
    locale: DescribeLocale,
    kind_label: String,
    name: Option<String>,
    facts: Vec<(&'static str, String)>,
}

impl LandmarkFacts {
    /// Reads `{locale, kind, kind_label, name?, facts}` as the host sends it.
    ///
    /// Facts outside [`DESCRIBED_FACTS`] are ignored, and a name or fact that is not a short
    /// single line of plain text is dropped rather than repaired.
    ///
    /// # Errors
    ///
    /// Returns [`DescribeInputError`] when the shape, kind or kind label is unreadable.
    pub fn from_json(json: &str) -> Result<Self, DescribeInputError> {
        let input: Input = serde_json::from_str(json).map_err(|_| DescribeInputError::Malformed)?;
        if !is_kind(&input.kind) {
            return Err(DescribeInputError::UnreadableKind);
        }
        let kind_label = plain(&input.kind_label).ok_or(DescribeInputError::UnreadableKindLabel)?;
        let name = input.name.as_deref().and_then(plain);
        let facts = DESCRIBED_FACTS
            .iter()
            .filter_map(|&key| {
                let value = match input.facts.get(key)? {
                    Value::String(text) => plain(text)?,
                    Value::Number(number) => number.to_string(),
                    _ => return None,
                };
                fact_value(key, &value).map(|value| (key, value))
            })
            .collect();
        Ok(Self {
            locale: input.locale,
            kind_label,
            name,
            facts,
        })
    }

    /// The landmark's name, when the archive gives one.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Tier 0: the description itself, worded from the facts and nothing else.
    #[must_use]
    pub fn template(&self) -> String {
        let mut text = match &self.name {
            Some(name) => format!("{name} — {}.", self.kind_label),
            None => format!("{}.", capitalized(&self.kind_label)),
        };
        for (key, value) in &self.facts {
            text.push(' ');
            text.push_str(&fact_sentence(self.locale, key, value));
        }
        text
    }
}

/// Whether `said` may be shown and kept in place of `facts.template()`.
///
/// Surrounding whitespace and the empty thinking block the pinned runtime writes are
/// ignored. Nothing else is repaired.
///
/// # Errors
///
/// Returns [`DescribeRefusal`] naming the first rule the answer breaks.
pub fn admit(facts: &LandmarkFacts, said: &str) -> Result<String, DescribeRefusal> {
    let text = without_empty_think_block(said).trim();
    let chars = text.chars().count();
    if chars == 0 || chars > DESCRIPTION_MAX_CHARS {
        return Err(DescribeRefusal::Length);
    }
    if text.contains(['<', '>', '{', '}', '`', '|']) || text.chars().any(char::is_control) {
        return Err(DescribeRefusal::Markup);
    }
    let ends = text
        .trim_end_matches(['"', '»', '”', ')'])
        .ends_with(['.', '!', '?', '…']);
    let sentences = sentence_ends(text);
    if !ends || sentences > MAX_SENTENCES {
        return Err(DescribeRefusal::Form);
    }
    let template = facts.template();
    let known = numbers(&template);
    if numbers(text).iter().any(|number| !known.contains(number)) {
        return Err(DescribeRefusal::NewNumber);
    }
    if facts.name().is_some_and(|name| !text.contains(name)) {
        return Err(DescribeRefusal::NameMissing);
    }
    let stems: Vec<String> = words(&template).map(stem).collect();
    let new_name = words(text).any(|word| {
        word.chars().next().is_some_and(char::is_uppercase)
            && !OPENERS.contains(&word)
            && !stems.contains(&stem(word))
    });
    if new_name {
        return Err(DescribeRefusal::NewName);
    }
    Ok(text.to_owned())
}

/// One fact as a sentence of its own.
fn fact_sentence(locale: DescribeLocale, key: &str, value: &str) -> String {
    match (locale, key) {
        (DescribeLocale::Uk, "start_date") => format!("З {value} року."),
        (DescribeLocale::Uk | DescribeLocale::Ru, "artist_name") => format!("Автор — {value}."),
        (DescribeLocale::Uk, "architect") => format!("Архітектор — {value}."),
        (DescribeLocale::Uk, "height") => format!("Висота — {value} м."),
        (DescribeLocale::Uk, _) => format!("Матеріал — {value}."),
        (DescribeLocale::Ru, "start_date") => format!("С {value} года."),
        (DescribeLocale::Ru, "architect") => format!("Архитектор — {value}."),
        (DescribeLocale::Ru, "height") => format!("Высота — {value} м."),
        (DescribeLocale::Ru, _) => format!("Материал — {value}."),
        (DescribeLocale::En, "start_date") => format!("Since {value}."),
        (DescribeLocale::En, "artist_name") => format!("By {value}."),
        (DescribeLocale::En, "architect") => format!("Architect: {value}."),
        (DescribeLocale::En, "height") => format!("{value} m tall."),
        (DescribeLocale::En, _) => format!("Made of {value}."),
    }
}

/// A fact's value as a description says it: a year for a date, plain metres for a height.
fn fact_value(key: &str, value: &str) -> Option<String> {
    match key {
        "start_date" => year(value),
        "height" => metres(value),
        _ => Some(value.to_owned()),
    }
}

/// The first run of exactly four digits.
fn year(value: &str) -> Option<String> {
    numbers(value).into_iter().find(|run| run.len() == 4)
}

/// A positive height in metres, with at most one decimal, without a unit.
fn metres(value: &str) -> Option<String> {
    let number = value
        .trim()
        .strip_suffix('m')
        .unwrap_or(value.trim())
        .trim();
    let (whole, fraction) = number.split_once('.').unwrap_or((number, ""));
    let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    if whole.is_empty() || whole.len() > 4 || !digits(whole) || !digits(fraction) {
        return None;
    }
    let whole = whole.trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    let tenth = fraction.chars().next().filter(|&digit| digit != '0');
    let metres = match tenth {
        Some(digit) => format!("{whole}.{digit}"),
        None => whole.to_owned(),
    };
    (metres != "0").then_some(metres)
}

/// The runs of letters in `text`.
fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_alphabetic())
        .filter(|word| !word.is_empty())
}

/// A word's first letters, lowercased: the same for its declined forms.
fn stem(word: &str) -> String {
    word.chars()
        .take(STEM_CHARS)
        .flat_map(char::to_lowercase)
        .collect()
}

/// How many sentences end in `text`: a terminator at the end or before a space, so a
/// decimal point is not one.
fn sentence_ends(text: &str) -> usize {
    let chars: Vec<char> = text.chars().collect();
    chars
        .iter()
        .enumerate()
        .filter(|&(at, c)| {
            matches!(c, '.' | '!' | '?' | '…')
                && chars.get(at + 1).is_none_or(|next| next.is_whitespace())
        })
        .count()
}

/// A short single line of plain text, trimmed, or nothing.
fn plain(value: &str) -> Option<String> {
    let text = value.trim();
    let readable = !text.is_empty()
        && text.chars().count() <= VALUE_MAX_CHARS
        && !text.chars().any(char::is_control)
        && !text.contains(['<', '>', '{', '}', '`', '|']);
    readable.then(|| text.to_owned())
}

/// Every run of ASCII digits, in order.
fn numbers(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_digit())
        .filter(|run| !run.is_empty())
        .map(str::to_owned)
        .collect()
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

fn is_kind(kind: &str) -> bool {
    let mut bytes = kind.bytes();
    kind.len() <= KIND_MAX_CHARS
        && bytes.next().is_some_and(|first| first.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte == b'_')
}

/// Drops the empty thinking block the pinned runtime writes once thinking is switched off.
fn without_empty_think_block(text: &str) -> &str {
    let trimmed = text.trim_start();
    trimmed
        .strip_prefix("<think>")
        .and_then(|rest| rest.trim_start().strip_prefix("</think>"))
        .unwrap_or(text)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_faculty_returns_an_actual_result_for_core_to_forward() {
        let request = r#"{"locale":"en","kind":"monument","kind_label":"monument","name":"Rodina","facts":{"start_date":"1981"}}"#;
        let plain = super::execute_description(request, None).expect("tier 0");
        assert_eq!(plain.text, "Rodina — monument. Since 1981.");
        assert_eq!(plain.produced_by, "template");
        assert_eq!(plain.template_version, 1);

        let retold = super::execute_description(
            request,
            Some(("Rodina has stood here since 1981.", "Qwen3-0.6B")),
        )
        .expect("model");
        assert_eq!(retold.produced_by, "Qwen3-0.6B");
        assert_eq!(retold.text, "Rodina has stood here since 1981.");

        let rejected = super::execute_description(
            request,
            Some(("Rodina was opened by Brezhnev in 1981.", "Qwen3-0.6B")),
        )
        .expect("fallback");
        assert_eq!(rejected, plain);
    }


    use serde_json::Value;

    use super::{DESCRIBE_SYSTEM_PROMPT, DESCRIPTION_MAX_CHARS, LandmarkFacts, admit};

    /// The fixture `nilx-one/web` tests its own port against.
    const SHARED_FIXTURE: &str = include_str!("../fixtures/landmark-description.json");

    fn fixture() -> Value {
        serde_json::from_str(SHARED_FIXTURE).expect("the fixture is JSON")
    }

    #[test]
    fn the_shared_fixture_carries_this_prompt_and_bound() {
        let fixture = fixture();
        assert_eq!(fixture["system_prompt"], DESCRIBE_SYSTEM_PROMPT);
        assert_eq!(fixture["max_chars"], DESCRIPTION_MAX_CHARS);
    }

    #[test]
    fn every_shared_case_is_worded_and_admitted_as_written() {
        let fixture = fixture();
        let cases = fixture["cases"].as_array().expect("cases");
        assert!(!cases.is_empty());
        for case in cases {
            let name = case["name"].as_str().expect("name");
            let facts = LandmarkFacts::from_json(&case["input"].to_string())
                .unwrap_or_else(|error| panic!("{name}: {error:?}"));
            assert_eq!(facts.template(), case["template"], "{name}");
            for said in case["accepts"].as_array().expect("accepts") {
                let said = said.as_str().expect("text");
                assert!(admit(&facts, said).is_ok(), "{name}: {said:?}");
            }
            for said in case["refuses"].as_array().expect("refuses") {
                let said = said.as_str().expect("text");
                assert!(admit(&facts, said).is_err(), "{name}: {said:?}");
            }
        }
    }

    #[test]
    fn every_shared_refused_input_is_refused() {
        let fixture = fixture();
        for case in fixture["refused_inputs"]
            .as_array()
            .expect("refused inputs")
        {
            let name = case["name"].as_str().expect("name");
            assert!(
                LandmarkFacts::from_json(&case["input"].to_string()).is_err(),
                "{name}"
            );
        }
    }

    #[test]
    fn an_admitted_answer_is_trimmed_of_the_empty_thinking_block() {
        let facts = LandmarkFacts::from_json(
            r#"{"locale":"en","kind":"statue","kind_label":"statue","name":"Rodina"}"#,
        )
        .expect("facts");
        assert_eq!(
            admit(&facts, "<think>\n\n</think>\n\nRodina stands here."),
            Ok("Rodina stands here.".to_owned())
        );
    }
}
