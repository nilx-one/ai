# © 2026 aiaiaiai · aiaiaiai.org
# SPDX-License-Identifier: MPL-2.0

# Avaia state contract

**Status:** v2 product contract (`AVAIA_STATE_SCHEMA_VERSION = 2`) for continuity across
inference-runtime replacement.

## Purpose

Avaia's inference runtime is disposable. Its weights, KV state, prompts, logits, embeddings,
and other model-specific artifacts are cache. They may disappear when a model is unloaded,
evicted, or replaced.

Product-owned state is different: it preserves Avaia's continuity without making a model
into the subject.

The contract (schema v2) stores only bounded product vocabulary:

- the current intent: her own target, an owner's navigation position, or her own curiosity;
- the last decision she made;
- the current pause reason.

There is no pending step. A decision is what Avaia does from the moment she makes it
(`AvaiaState::decide`); nothing admits or approves it. `decide` takes a `MenuDecision`, which
only `DecisionMenu::decide` makes, so what she carries out was always one of the offered
choices. The stored `last_decision` is the `AvaiaDecision` record of it: readable, and never
something `decide` accepts back. An owner's navigation position
becomes what she heads for (`AvaiaState::head_for`) and, once she arrives, she is back to her
own choices (`AvaiaState::arrive`).

It does not store a completion record. Whether an interaction happened remains owned by
0x1/core and its BondChain semantics.

## Contract

The Rust type is AvaiaState in src/state.rs.

| Field | Meaning | Durable | Model-specific |
|---|---|---:|---:|
| schema_version | state schema revision | yes | no |
| intent | what Avaia is currently trying to do | yes | no |
| last_decision | latest decision Avaia made; an owner's position is not one | yes | no |
| pause_reason | why the current intent is paused | yes | no |

AvaiaState::model_context() is the only model-facing projection defined here. It contains
the same bounded product vocabulary and does not expose raw journal rows, prompts, embeddings,
KV state, logits, or model-specific cache.

## What is deliberately absent

**Presence history.** The journal records where a person stood. It is device-local and is not
folded into Avaia identity or continuity state.

**Interaction completion.** A decision is not an interaction and a pause is not a failed
interaction. Whether an interaction happened belongs to the protocol boundary; where Avaia
walks never reaches it.

**Permission.** There is none to record. No field holds an approval, a delegation scope or a
step waiting to be admitted, because a walk has none of them.

**Relationship/BondChain state.** Neither is mirrored here.

**Model identity.** Replacing a model or runtime cannot mint a different Avaia subject.

**Unbounded history.** The state is not a growing transcript. If future product behavior needs
more continuity, it must add an explicit bounded field with its own semantics.

## Persistence and authentication boundary

Storage is this repository's trust boundary, while authentication and subject identity remain
owned by the identity/protocol layer. The crate still does no I/O: `src/store.rs` states the
rules as a port, `AvaiaStateStore`, that a product implements over whatever storage it has,
and two functions, `restore` and `persist`, that are the only way through it.

| Rule | Where it holds |
|---|---|
| bind the stored value to the already-established Avaia subject | a record is `StoredAvaiaState { subject, state }`; `restore` and `persist` refuse a record bound to another subject (`WrongSubject`) and never read it as hers |
| preserve the schema version | `persist` writes only the current schema (`NotCurrentSchema`) |
| reject or quarantine a major schema it cannot interpret rather than rewriting it | `restore` reads a v1 record and upgrades it; a newer one is `Restored::Quarantined`, she starts fresh, and `persist` refuses to write over it (`WouldOverwriteNewer`) |
| replace the state atomically | `AvaiaStateStore::replace` is atomic by contract: after a failed write the previous record is still the one kept |
| never derive subject identity from model identity | `AvaiaSubject` is an opaque, non-empty identifier handed in by the identity layer; nothing in the crate makes one from a model, runtime, device or cache |

What authenticates the subject, and where the bytes live (IndexedDB, a file, a server row),
stay outside this crate. A storage implementation keeps bytes; it does not decide what is
read or written.

## Schema v2

v2 renamed `last_proposal` to `last_decision`, `OwnerControl` to `OwnerAtWheel` and
`NoAdmissibleAction` to `NothingToChoose`, and added the `OwnerWaypoint` intent. A v1 record
reads field for field under the old names; `AvaiaState::upgraded` returns it as v2, and
returns nothing for a schema newer than this crate, which a store keeps as it is.

## Runtime replacement

The runtime lifecycle and the state lifecycle are independent.

Replacing or evicting the model may discard model cache without changing AvaiaState.
Restoring a new runtime consumes the existing state; it does not create a new subject.
`restore` takes the subject, never a model: the same subject on another runtime, another model
size, or after the model was evicted from a device's cache gets back the same state. The two
lifecycles stay unrelated: evicting a model never touches the store, and losing the store
never touches a cached model.

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
