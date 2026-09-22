# © 2026 aiaiaiai · aiaiaiai.org
# SPDX-License-Identifier: MPL-2.0

# Avaia state contract

**Status:** v1 product contract for continuity across inference-runtime replacement.

## Purpose

Avaia's inference runtime is disposable. Its weights, KV state, prompts, logits, embeddings,
and other model-specific artifacts are cache. They may disappear when a model is unloaded,
evicted, or replaced.

Product-owned state is different: it preserves Avaia's continuity without making a model
into the subject.

The v1 contract stores only bounded product vocabulary:

- the current intent;
- the last action proposal;
- the current pause reason.

It does not store a completion record. Whether an interaction happened remains owned by
0x1/core and its BondChain semantics.

## Contract

The Rust type is AvaiaState in src/state.rs.

| Field | Meaning | Durable | Model-specific |
|---|---|---:|---:|
| schema_version | state schema revision | yes | no |
| intent | what Avaia is currently trying to do | yes | no |
| last_proposal | latest proposal emitted by Avaia | yes | no |
| pause_reason | why the current intent is paused | yes | no |

AvaiaState::model_context() is the only model-facing projection defined here. It contains
the same bounded product vocabulary and does not expose raw journal rows, prompts, embeddings,
KV state, logits, or model-specific cache.

## What is deliberately absent

**Presence history.** The journal records where a person stood. It is device-local and is not
folded into Avaia identity or continuity state.

**Interaction completion.** A proposal is not an interaction and a pause is not a failed
interaction. Admission and completion belong to the protocol boundary.

**Relationship/BondChain state.** Neither is mirrored here.

**Model identity.** Replacing a model or runtime cannot mint a different Avaia subject.

**Unbounded history.** The state is not a growing transcript. If future product behavior needs
more continuity, it must add an explicit bounded field with its own semantics.

## Persistence and authentication boundary

The persistence adapter is intentionally not part of this crate's v1 type. Storage is this
repository's trust boundary, while authentication/subject identity remains owned by the
identity/protocol layer.

A storage implementation must:

1. bind the stored value to the already-established Avaia subject;
2. preserve the schema version;
3. reject or quarantine a major schema it cannot interpret rather than rewriting it;
4. replace the state atomically;
5. never derive subject identity from model identity.

## Runtime replacement

The runtime lifecycle and the state lifecycle are independent.

Replacing or evicting the model may discard model cache without changing AvaiaState.
Restoring a new runtime consumes the existing state; it does not create a new subject.

## Future evolution

A new state field is a contract change, not an implementation convenience. It must answer:

- what product fact the field represents;
- why it is independent of the model;
- its bounded size and lifecycle;
- whether it is safe to expose through model_context();
- why it is not protocol completion or Relationship evidence.

No field should be added merely because a model can remember or derive it.

---

© 2026 aiaiaiai · aiaiaiai.org
