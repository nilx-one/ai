// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0

//! Routing one failure to the two places that need it at once.
//!
//! A failure that reaches only an operator leaves the person waiting on the operation with
//! nothing; one that reaches only that person leaves nobody able to see it later. So a
//! single value goes two ways: a [`FailureRecord`] to a [`FailureSink`], and an
//! [`AvaiaFailureReport`] to whatever renders it.
//!
//! Neither direction happens here. This crate writes nothing, sends nothing and renders
//! nothing — the store is a port the host implements, and the sentence a person reads
//! belongs to the client that owns locale and tone.

use aiai_runtime::prelude::{
    DecimalU64, ErrorCode, FailureKind, FailureRecord, FoundationError, OperationId, SessionId,
};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

/// Where a product keeps the failures it observed.
///
/// The eventual store is one Supabase table collecting failures from everywhere, which is
/// I/O, asynchronous, and remote — none of which this crate may acquire without breaking
/// the port model it is built on and the browser target that follows from it. So the sink
/// is a trait, implemented by whatever hosts this runtime.
///
/// What is owed to that table is the row's shape, which is [`FailureRecord`]'s, and its
/// bytes, which are canonical JSON — so the row a host writes is what every other binding
/// of this contract decodes.
pub trait FailureSink {
    /// What this sink reports when it could not keep a record.
    type Error;

    /// Hands one record to the store.
    ///
    /// `Ok(())` is the sink's own report that it accepted the record. It is not evidence
    /// that a row exists, and nothing here may be read as such.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the record could not be kept. See [`route_failure`]
    /// for what that does, and does not, do to the failure being reported.
    fn record(&mut self, record: &FailureRecord) -> Result<(), Self::Error>;
}

/// One failure in the shape the Web client renders it from.
///
/// The client shows a failure as a toast and must not re-derive what kind of failure it is
/// holding: the classification belongs to the foundation, and a second copy of it in the
/// client would be a copy that drifts. It also cannot re-derive it today, because the
/// foundation's client-side package is unpublished. So the kind travels already computed.
///
/// The foundation keeps the kind out of its own envelopes precisely so that a code and a
/// kind can never disagree on a wire. Carrying it here is the deliberate exception, and it
/// is paid for on both entrances: only [`AvaiaFailureReport::for_failure`] builds one, and
/// a decoded report whose kind or retryability contradicts its code is refused rather than
/// believed.
///
/// It carries no subject identifier — the record it travels beside omits one for a
/// retention reason a client-facing copy would defeat — and no message. What a person
/// reads, in which language, is the client's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AvaiaFailureReport {
    code: ErrorCode,
    kind: FailureKind,
    retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation_id: Option<OperationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<SessionId>,
}

impl AvaiaFailureReport {
    /// Builds the report for `error`, classifying it here so that a caller cannot.
    ///
    /// The kind and retryability are read off the code, never supplied: a caller able to
    /// pass them could describe a withheld decision as a malfunction, or invite a retry of
    /// something that fails identically every time.
    #[must_use]
    pub fn for_failure(error: &FoundationError, session_id: Option<SessionId>) -> Self {
        let code = error.code();
        Self {
            code,
            kind: code.kind(),
            retryable: code.is_retryable(),
            operation_id: error.operation_id().cloned(),
            session_id,
        }
    }

    /// Returns the stable foundation error code.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// Returns what kind of failure the code names.
    #[must_use]
    pub const fn kind(&self) -> FailureKind {
        self.kind
    }

    /// Returns whether the same operation, unchanged, may succeed if attempted again.
    #[must_use]
    pub const fn retryable(&self) -> bool {
        self.retryable
    }

    /// Returns the correlation handle the failure carried, when it carried one.
    #[must_use]
    pub const fn operation_id(&self) -> Option<&OperationId> {
        self.operation_id.as_ref()
    }

    /// Returns the session the failure happened in, when one existed yet.
    #[must_use]
    pub const fn session_id(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }
}

/// The decoded form, before it has been checked against its own code.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DecodedReport {
    code: ErrorCode,
    kind: FailureKind,
    retryable: bool,
    #[serde(default)]
    operation_id: Option<OperationId>,
    #[serde(default)]
    session_id: Option<SessionId>,
}

impl<'de> Deserialize<'de> for AvaiaFailureReport {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let decoded = DecodedReport::deserialize(deserializer)?;
        if decoded.kind != decoded.code.kind() || decoded.retryable != decoded.code.is_retryable() {
            return Err(D::Error::custom(
                "failure report contradicts the classification of its own error code",
            ));
        }
        Ok(Self {
            code: decoded.code,
            kind: decoded.kind,
            retryable: decoded.retryable,
            operation_id: decoded.operation_id,
            session_id: decoded.session_id,
        })
    }
}

/// Both destinations of one failure, after [`route_failure`] has fanned it out.
#[derive(Debug)]
#[must_use]
pub struct RoutedFailure<E> {
    /// What the client renders. Built whether or not the sink kept anything.
    pub report: AvaiaFailureReport,
    /// What the sink reported about keeping the record. Returned rather than resolved,
    /// because a store that could not answer is a fact of its own and this crate has no
    /// second store to try.
    pub recorded: Result<(), E>,
}

/// Routes one failure: a row to `sink`, a report back to the caller.
///
/// `recorded_at_unix_ms` is read from the product's `Clock` port before this call. This
/// crate reads no clock, and a moment that could not be obtained is the caller's own
/// outcome rather than a substituted value.
///
/// # A failing sink does not consume the failure
///
/// Recording is best-effort for the caller's flow: a sink error is returned in
/// [`RoutedFailure::recorded`], never in place of the report. Returning
/// `Result<AvaiaFailureReport, S::Error>` instead would let an unrelated storage fault
/// replace the failure the person is actually waiting on — telling them the wrong thing
/// went wrong, which is the one outcome this whole taxonomy exists to prevent. It is not
/// swallowed either: the sink's error comes back for the caller to surface wherever it
/// surfaces operator-facing problems.
pub fn route_failure<S: FailureSink>(
    sink: &mut S,
    recorded_at_unix_ms: DecimalU64,
    session_id: Option<SessionId>,
    error: &FoundationError,
) -> RoutedFailure<S::Error> {
    let record = FailureRecord::new(recorded_at_unix_ms, session_id.clone(), error.clone());
    let recorded = sink.record(&record);
    RoutedFailure {
        report: AvaiaFailureReport::for_failure(error, session_id),
        recorded,
    }
}

#[cfg(test)]
mod tests {
    use aiai_runtime::contracts::canonical_json;
    use aiai_runtime::prelude::{
        CapabilityName, DecimalU64, ErrorCode, FailureKind, FailureRecord, FoundationError,
        OperationId, ProposalId, SessionId,
    };

    use super::{AvaiaFailureReport, FailureSink, route_failure};

    const RECORDED_AT: DecimalU64 = DecimalU64::new(1_767_225_600_000);

    /// A sink reports what it could keep and nothing about what happened afterwards.
    #[derive(Debug, PartialEq, Eq)]
    struct SinkUnavailable;

    #[derive(Default)]
    struct CollectingSink {
        kept: Vec<FailureRecord>,
    }

    impl FailureSink for CollectingSink {
        type Error = SinkUnavailable;

        fn record(&mut self, record: &FailureRecord) -> Result<(), Self::Error> {
            self.kept.push(record.clone());
            Ok(())
        }
    }

    struct RefusingSink;

    impl FailureSink for RefusingSink {
        type Error = SinkUnavailable;

        fn record(&mut self, _record: &FailureRecord) -> Result<(), Self::Error> {
            Err(SinkUnavailable)
        }
    }

    fn session_id() -> SessionId {
        format!("ses_{}", "a".repeat(32))
            .parse()
            .expect("canonical session id")
    }

    fn operation_id() -> OperationId {
        format!("op_{}", "b".repeat(32))
            .parse()
            .expect("canonical operation id")
    }

    fn proposal_id() -> ProposalId {
        format!("prp_{}", "c".repeat(32))
            .parse()
            .expect("canonical proposal id")
    }

    /// One code per kind, so a classification that drifts fails here rather than in a toast.
    fn one_failure_of_every_kind() -> Vec<FoundationError> {
        vec![
            FoundationError::inference_unavailable(Some(operation_id())),
            FoundationError::authority_withheld(
                Some(operation_id()),
                proposal_id(),
                "message".parse::<CapabilityName>().expect("canonical name"),
            ),
            FoundationError::runtime_inactive(Some(operation_id())),
            FoundationError::malformed_envelope(Some(operation_id())),
            FoundationError::sequence_exhausted(Some(operation_id())),
        ]
    }

    #[test]
    fn the_sink_receives_the_record_the_foundation_would_build() {
        let error = FoundationError::inference_unavailable(Some(operation_id()));
        let mut sink = CollectingSink::default();

        let routed = route_failure(&mut sink, RECORDED_AT, Some(session_id()), &error);

        assert_eq!(routed.recorded, Ok(()));
        assert_eq!(
            sink.kept,
            vec![FailureRecord::new(
                RECORDED_AT,
                Some(session_id()),
                error.clone()
            )]
        );
        assert_eq!(
            routed.report,
            AvaiaFailureReport::for_failure(&error, Some(session_id()))
        );
    }

    #[test]
    fn a_refusing_sink_does_not_take_the_failure_with_it() {
        let error = FoundationError::authority_withheld(
            Some(operation_id()),
            proposal_id(),
            "message".parse::<CapabilityName>().expect("canonical name"),
        );

        let routed = route_failure(&mut RefusingSink, RECORDED_AT, Some(session_id()), &error);

        assert_eq!(routed.recorded, Err(SinkUnavailable));
        assert_eq!(routed.report.code(), ErrorCode::AuthorityWithheld);
        assert_eq!(routed.report.kind(), FailureKind::Withheld);
        assert_eq!(routed.report.operation_id(), Some(&operation_id()));
    }

    #[test]
    fn the_report_classifies_exactly_as_the_foundation_does() {
        for error in one_failure_of_every_kind() {
            let report = AvaiaFailureReport::for_failure(&error, None);
            assert_eq!(report.kind(), error.code().kind());
            assert_eq!(report.retryable(), error.code().is_retryable());
        }
    }

    /// A withheld admission is a decision, not a malfunction, and repeating it asks the
    /// same authority the same question. An unreachable model decided nothing at all.
    #[test]
    fn a_decision_is_not_retryable_and_an_unreachable_port_is() {
        let withheld = AvaiaFailureReport::for_failure(
            &FoundationError::authority_withheld(
                None,
                proposal_id(),
                "message".parse::<CapabilityName>().expect("canonical name"),
            ),
            None,
        );
        assert_eq!(withheld.kind(), FailureKind::Withheld);
        assert!(!withheld.retryable());

        let unavailable =
            AvaiaFailureReport::for_failure(&FoundationError::inference_unavailable(None), None);
        assert_eq!(unavailable.kind(), FailureKind::Unavailable);
        assert!(unavailable.retryable());
    }

    #[test]
    fn every_kind_is_represented_exactly_once() {
        let kinds: Vec<FailureKind> = one_failure_of_every_kind()
            .iter()
            .map(|error| error.code().kind())
            .collect();

        assert_eq!(
            kinds,
            vec![
                FailureKind::Unavailable,
                FailureKind::Withheld,
                FailureKind::Gated,
                FailureKind::Rejected,
                FailureKind::Exhausted,
            ]
        );
    }

    #[test]
    fn the_report_round_trips_through_canonical_json() {
        for error in one_failure_of_every_kind() {
            let report = AvaiaFailureReport::for_failure(&error, Some(session_id()));
            let encoded = canonical_json(&report).expect("canonical JSON");
            let decoded: AvaiaFailureReport =
                serde_json::from_slice(&encoded).expect("a report this crate wrote decodes");

            assert_eq!(decoded, report);
            assert_eq!(canonical_json(&decoded).expect("canonical JSON"), encoded);
        }
    }

    #[test]
    fn the_record_round_trips_through_canonical_json() {
        let mut sink = CollectingSink::default();
        for error in one_failure_of_every_kind() {
            let _ = route_failure(&mut sink, RECORDED_AT, Some(session_id()), &error);
        }

        for record in &sink.kept {
            let encoded = canonical_json(record).expect("canonical JSON");
            let decoded: FailureRecord =
                serde_json::from_slice(&encoded).expect("a record this crate built decodes");

            assert_eq!(&decoded, record);
            assert_eq!(canonical_json(&decoded).expect("canonical JSON"), encoded);
        }
    }

    /// The one thing carrying a derived value on a wire risks: a payload that disagrees
    /// with itself. It is refused rather than believed.
    #[test]
    fn a_report_contradicting_its_own_code_is_refused() {
        let contradiction =
            br#"{"code":"authority_withheld","kind":"unavailable","retryable":true}"#;

        assert!(serde_json::from_slice::<AvaiaFailureReport>(contradiction).is_err());
    }
}
