# © 2026 aiaiaiai · aiaiaiai.org
# SPDX-License-Identifier: MPL-2.0

# Presence journal training-signal egress policy

**Status:** Phase 1 policy. No journal-derived training route is authorized by this
document.

## Policy

The presence journal is device-local evidence. Its default and current egress policy is
**no egress**: raw records, transformed records, aggregates, embeddings, summaries, and
model-derived artifacts whose meaning depends on journal records must not leave the device
for training, evaluation, telemetry, analytics, or model improvement.

This policy does not make journal records protocol truth. A presence record is not evidence
of a Bond interaction, consent, mutuality, Relationship state, or BondChain formation.

## Allowed boundary

No Phase 1 or Phase 2 implementation may upload journal records implicitly.

A future egress route can exist only after a separate, explicit authorization defines all of:

1. **Purpose** — the exact use is named before collection or transfer.
2. **Consent** — the person explicitly opts in to that purpose; silence, product use,
   model availability, or device pairing is not consent.
3. **Data boundary** — the route names whether raw records, transformed records, or
   aggregates are permitted. Data not required for the stated purpose stays local.
4. **Provenance** — every exported artifact records the source class, transformation,
   purpose, and authorization that permitted the export.
5. **Retention** — the maximum retention period and deletion mechanism are explicit.
6. **Revocation** — withdrawal stops future exports and defines what deletion is possible
   for already-exported material.
7. **Derived artifacts** — a policy explicitly states whether artifacts derived before
   revocation may remain. They are not implicitly exempt from revocation.
8. **Auditability** — export is an observable, narrowly scoped operation, never a side
   effect of rendering, narration, inference, synchronization, or cache warming.

## Training-specific prohibition

The following are prohibited until a future training-egress policy explicitly authorizes
them:

- uploading raw presence-journal rows;
- uploading a transformed or aggregated representation of those rows;
- turning journal history into embeddings or other training artifacts and uploading them;
- silently including journal-derived text in a general telemetry or model-improvement stream;
- retaining an exported training artifact after source revocation merely because it is
  considered "derived";
- using journal movement history as Bond, Relationship, consent, mutuality, interaction, or
  BondChain evidence.

## Relationship and protocol boundary

The AI runtime may consume local journal evidence for a product feature when that feature's
contract permits it. Such consumption does not promote the evidence into protocol truth.

In particular:

- a visit is not an interaction;
- movement is not reciprocity;
- narration is not an observation authority;
- a model output is a proposal, not a completed interaction;
- a journal-derived cache is not a BondChain record.

The authoritative interaction and Relationship semantics remain outside this repository.

## Implementation rule

Any future code that could cross this boundary must have an explicit egress capability in
its contract. There must be no generic "send telemetry" path that can receive journal
payloads by structural compatibility alone.

The absence of an egress capability is the Phase 1 safety property.

## Review gate

Before any aggregation, training, telemetry, or model-improvement implementation using
journal-derived data is merged, this policy must be revisited and the purpose, consent,
boundary, provenance, retention, revocation, and derived-artifact rules must be approved
as part of that implementation.

---

© 2026 aiaiaiai · aiaiaiai.org
