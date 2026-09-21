# Avaia walks the map, only while spectating

**Status:** directional, non-normative. Normative AI Bond identity, authority, activation, and
BondChain semantics remain owned by [AI Bonds](https://github.com/nilx-one/0x1/blob/master/documents/04-ai-bonds.md),
the [Protocol Laws](https://github.com/nilx-one/0x1/blob/master/documents/00-protocol-laws.md),
and [Map Architecture](https://github.com/nilx-one/0x1/blob/master/documents/12-map-architecture.md).
This document states a product direction and names what still has to be decided elsewhere; it
does not itself change `0x1` or `core`.

## Purpose

"Avaia walks the map" is not one feature. It is three narrower ones that happen to share a
camera:

```text
movement       -> a sequence of proposed, admitted NavigateTo steps
observation    -> a local record of what was already visible while moving
discovery      -> something on the map worth walking toward
```

This document scopes a first slice of the first two, against the map projections that already
exist, and states what the third needs before it can exist at all. It deliberately excludes
transport: routing, vehicles, and transit state are a separate, larger capability with their
own authority and rendering questions, already named as out of scope for basemap work in
[`nilx-one/web`'s map data doc](https://github.com/nilx-one/web/blob/main/docs/map-data.md).
Nothing here blocks on transport, and this workstream must not grow to include it.

## What already exists

Nothing below is new. The gap is that none of it has been asked to run in sequence yet.

| Piece | Where | What it does |
|---|---|---|
| A target-bound decision | `src/decision.rs` — `DecisionMenu` | Avaia chooses among `MapTargetId`s a caller already resolved; it cannot mint one. Grammar and parse enforce this twice. |
| A movement proposal | `src/lib.rs` — `AvaiaActionProposal::NavigateTo{target}` / `StopNavigation` | The shape that would go into `RuntimeSession::propose_candidates`. Still a proposal — admission is a separate boundary. |
| The autonomy gate | `src/lib.rs` — `AvaiaControlMode::Spectate -> ActivationState::Active` | The only mode in which the runtime is active at all. Manual quiesces; Offline is dormant. |
| A reserved observation slot | `nilx-one/core` `docs/presence-journal.md`; mirrored (not yet widened) in `nilx-one/web` `packages/presence-contract/src/index.ts` | `VisitRecord.source` names `"avaia"` as reserved for exactly this later phase. The TypeScript type today only admits `"self"`. |
| The fog | `nilx-one/web` `packages/map-shade`, `packages/presence-idb` (`ShadeSource.litCells()`) | Already computes lit-cell membership from the journal. Nothing new needed to know what is "not in fog." |
| The public map state | `nilx-one/0x1` `documents/12-map-architecture.md` — `map.registry` | `cell_activity`, `physical_presences[]`, `digital_presence?`. No landmark concept, no transport. |

## Design: movement and observation

### 1. A walk is a chain of the existing proposal, nothing new

Each step is: world layer resolves candidate `MapTargetId`s for the current position (adjacent
cells, or discoverable targets once §3 exists) → `DecisionMenu` → Avaia picks `NavigateTo` or
`StopNavigation` → admission → effect. No new proposal type. Landmarks, once they exist, are
just another kind of thing a `MapTargetId` can refer to.

### 2. The gate stays exactly SPECTATE, and nothing wider

`AvaiaControlMode::Spectate` is already the only mode that reaches `ActivationState::Active`.
This workstream adds no second gate and no background-life path: leaving SPECTATE mid-walk
quiesces it the same way it quiesces anything else running.

What is genuinely open, and belongs to the authority/delegation contract rather than to this
document: whether each `NavigateTo` step needs its own admission, or whether SPECTATE grants a
bounded delegation scope for "a walk of up to N steps" that is admitted once. `0x1`'s current
text says only that authorization is bounded by contract and is not unlimited autonomy
(`device-runtime-and-control.md`, DRC6) — it does not yet say which of these two shapes a walk
takes. This document does not resolve it.

### 3. Observation is a cache until someone decides to make it durable

Per [`mind.md`](mind.md): a value whose meaning depends on which model produced it is a cache,
not state. "What Avaia saw" rendered live during a walk is exactly that — per-device,
disposable, never synced.

The one place this workstream reuses rather than invents is the reserved
`VisitRecord.source: "avaia"` slot. Persisting a walk step there gives Avaia a durable local
memory of where it has already been, on the same terms `"self"` records already carry:
device-local, encrypted at rest, never synced, never exported, never training signal without a
separate governance contract (`presence-journal-lifecycle.md`, the `artificial-bonds/training-
signal.md` egress gate). Widening the TypeScript union from `"self"` to `"self" | "avaia"` is
the only concrete code change this implies, and it is client-side only.

### 4. "What's new" is a diff against the caller's own last look

Because `map.registry` is reconstructable public state, "new since Avaia last looked at this
cell" is a local diff: the current projection for a lit cell against whatever this device last
cached for it. No protocol change is required to compute this over the two projections that
exist today (`physical_presences[]`, `digital_presence?`). It becomes a richer diff the moment
§5 exists, and not before.

## Design: what discovery needs that does not exist yet

A walk with nothing to walk toward is a demo, not a product. `map.registry` today carries only
business presence. "Визначні місця" — landmarks, authored points of interest that fill a city
independently of who is transacting where — are not represented anywhere in the current
contract.

This is protocol-level work. `nilx-one/ai` and `nilx-one/web` can consume such a projection;
neither can create it, and this document does not attempt to. The shape to follow already
exists in the specification, under "Future Creator Projections"
(`12-map-architecture.md`): authored discovery state that is placed on the map without
being business presence, without implying a Bond's physical location, and without proving
attendance, delivery, or BondChain formation. A landmark projection is that same pattern
applied to authored points of interest instead of creator offers.

Minimum shape Avaia's walk would need from it, stated as requirements rather than as a wire
format — the wire format is `0x1`'s to define:

- a landmark resolves inside a cell, the way a presence marker does;
- a landmark carries no claim of ownership, attendance, or BondChain by any Bond unless a
  separate contract says otherwise, mirroring invariants 11–14 already written for creator
  projections;
- a landmark has its own versioned lifecycle (created / updated / retired) that "new" is
  computed against, so two devices' local diffs do not silently disagree about what counts as
  new;
- authorship is decided before publication, not inferred from this workstream: whether
  landmarks are operator-curated seed content, creator-authored in the existing sense, or
  something else is an open product question (see below), not a default this document picks.

Until a versioned landmark projection contract lands in `nilx-one/0x1` and is implemented in
`nilx-one/core`, the correct implementation here is: no landmark data, same as the specification
already says for creator projections not yet published (`12-map-architecture.md` invariant 14).

## Explicitly out of scope for this workstream

- **Transport.** Named separately in `map-data.md`; large enough to need its own workstream and
  its own authority questions once it exists.
- **Autonomy outside SPECTATE.** No background walking, no persistence of intent across a mode
  change.
- **Training-signal egress** from any walk-derived observation.
- **A protocol change implemented from this repository.** The landmark projection is drafted
  and reviewed in `nilx-one/0x1`; this repository only states what it would consume.

## Open questions

Stated so this document does not silently pick an answer by omission:

1. Per-step admission versus a bounded walk-delegation scope (§2).
2. Who authors landmarks, and under what moderation and expiry rules — the same open items
   `12-map-architecture.md` already lists for creator projections, applied to a curated-content
   case instead of a creator-authored one.
3. Whether a landmark projection is versioned independently of `physical_presences[]` /
   `digital_presence?` or folds into a fourth field of the same `map.registry` shape.

## Invariants

1. **MWL1.** A walk is a sequence of the existing `NavigateTo` / `StopNavigation` proposal;
   this workstream introduces no second action type for movement.
2. **MWL2.** Only `AvaiaControlMode::Spectate` reaches the activation state a walk can run in.
3. **MWL3.** Avaia never mints a `MapTargetId`; the world layer resolves candidates before a
   decision is offered.
4. **MWL4.** Walk-derived observation is device-local by default; persisting it uses the
   already-reserved `source: "avaia"` slot and no other channel.
5. **MWL5.** No walk-derived data is synced, exported, or used as training signal absent a
   separate governance contract.
6. **MWL6.** No landmark, artifact, or point-of-interest data is published or consumed before a
   versioned projection contract exists in `nilx-one/0x1`.
7. **MWL7.** Transport is not part of this workstream under any of its steps.

## Related

- [The implementation plan](implementation-plan.yaml)
- [One mind, many models](mind.md)
- [What belongs upstream, and what does not](foundation-gaps.md)
- [Choosing the models Avaia loads](model-selection.md)
- [`nilx-one/core` — Presence journal](https://github.com/nilx-one/core/blob/main/docs/presence-journal.md)
- [`nilx-one/0x1` — Map Architecture](https://github.com/nilx-one/0x1/blob/master/documents/12-map-architecture.md)
- [`nilx-one/0x1` — Device Runtime and Control](https://github.com/nilx-one/0x1/blob/master/documents/artificial-bonds/device-runtime-and-control.md)
- [`nilx-one/web` — Map data](https://github.com/nilx-one/web/blob/main/docs/map-data.md)
- [`nilx-one/web` — Presence journal lifecycle](https://github.com/nilx-one/web/blob/main/docs/presence-journal-lifecycle.md)

---

© 2026 aiaiaiai · aiaiaiai.org
