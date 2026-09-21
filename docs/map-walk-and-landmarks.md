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

Each step is: world layer resolves candidate `MapTargetId`s for the current position → `DecisionMenu`
→ Avaia picks `NavigateTo` or `StopNavigation` → admission → effect. No new proposal type.
Landmarks, once they exist, are just another kind of thing a `MapTargetId` can refer to. §2
states where those candidates may come from at Stage 1.

### 2. Stage 1 is bounded by what is already lit

The fog mechanic already has an owner: a person's own device observations, mirrored through
`presence-geo`'s tracker, light a cell only after a real dwell (`presence-contract`'s
`PRESENCE_DWELL_MS`, `PRESENCE_ACCURACY_GATE_M`). Stage 1 does not touch that mechanic or
duplicate it. It only says where Avaia may stand and move inside it:

- **Avaia's position originates at the person's current cell.** The character appears where the
  owner's own last accepted observation placed them — not at an arbitrary point, and not
  requiring a separate spawn contract.
- **The candidate set for every `DecisionMenu` is `ShadeSource.litCells()` and nothing past its
  boundary.** The world layer resolving `MapTargetId`s (§1) must intersect its candidates with
  the already-lit set before Avaia ever sees them. A fogged (grey) cell is not merely a target
  Avaia would be refused for choosing — it is never constructed as a candidate at all.
- **A mark — a landmark once one exists (see "what discovery needs" below), or any other point
  Avaia's walk could reference — may only be placed inside a lit cell**, for the same reason:
  the grey zone has no confirmed geography for this device to reason about yet.
- **Avaia's own movement must not light new cells.** This is not free to assume: `map-shade`'s
  `createShadeSource` lights whatever cell a new `VisitRecord` names, regardless of `source`
  (`presence-idb/src/index.ts`, the `store.subscribe` callback in `createShadeSource`). Because
  every candidate is already lit by construction (the point above), an admitted `NavigateTo`
  step can only ever write a `source: "avaia"` record into a cell that was lit before the step
  was offered — so this invariant holds by construction, not by a special case added to
  `map-shade`. It is stated here because it would silently stop holding if a later change ever
  let candidate resolution reach past the lit boundary.

Only the person's own movement expands the lit boundary at Stage 1. Whether a later stage lets
Avaia's own walk reveal fog is an explicit, separate product decision — not a default this
document or its code picks by omission.

### 3. The gate stays exactly SPECTATE, and nothing wider

`AvaiaControlMode::Spectate` is already the only mode that reaches `ActivationState::Active`.
This workstream adds no second gate and no background-life path: leaving SPECTATE mid-walk
quiesces it the same way it quiesces anything else running.

What is genuinely open, and belongs to the authority/delegation contract rather than to this
document: whether each `NavigateTo` step needs its own admission, or whether SPECTATE grants a
bounded delegation scope for "a walk of up to N steps" that is admitted once. `0x1`'s current
text says only that authorization is bounded by contract and is not unlimited autonomy
(`device-runtime-and-control.md`, DRC6) — it does not yet say which of these two shapes a walk
takes. This document does not resolve it.

### 4. Observation is a cache until someone decides to make it durable

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

### 5. "What's new" is a diff against the caller's own last look

Because `map.registry` is reconstructable public state, "new since Avaia last looked at this
cell" is a local diff: the current projection for a lit cell against whatever this device last
cached for it. No protocol change is required to compute this over the two projections that
exist today (`physical_presences[]`, `digital_presence?`). It becomes a richer diff the moment
a landmark projection exists, and not before.

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

### `art_register`: where landmark content comes from, kept apart from where it becomes truth

The protocol projection above answers what a landmark *is*, once it is allowed to exist. It
does not answer where its content comes from before anyone has proposed a single entry. That
content side is a separate, smaller piece this workstream can scope now, because it commits
`0x1` to nothing: `art_register` is a product-owned content register, not protocol truth, and
nothing in it reaches a person, a client's map state, or `DecisionMenu` until it has crossed
the landmark-projection gate above.

Two sources feed it:

- **OSM extraction.** The basemap archive already carries a `pois` source layer
  (`nilx-one/web` `docs/map-data.md`) — the published style reads no attributes from it today
  (`deploy/web/map/0.1.0/style.json`'s `pois` layer is a bare circle keyed only on
  `source-layer`), so its declared fields are unconfirmed rather than assumed. The existing
  discipline in this codebase already refuses to guess: `deploy/web/inspect-basemap.sh` prints
  what the real archive actually declares, and a filter on a field it does not carry "quietly
  disappears" rather than erroring (`map-data.md`). Building `art_register` starts by running
  that inspection against the deployed archive and reading its `pois` fields off the output —
  not off generic OpenStreetMap tagging documentation — before any allowlist (candidate tags:
  `historic=*`, `tourism=attraction|artwork|viewpoint|museum`, `memorial=*`, `heritage=*`) is
  written down as fact.
- **Manual entries.** Additions, corrections, and removals a person curates directly. This
  workstream does not reuse `BondArtificialPositionSettings`'s disclosure-override code or its
  storage — that feature answers a different question, who sees a Bond's declared position —
  but its map-point-selection interaction (`MapPointSelection` / `MapRenderer`, "tap a point on
  the map") is the proven pattern to build a landmark-entry editor against, rather than a new
  interaction invented for this.

`art_register` is a build-time or admin-time artifact, not a live query against OSM or any
third-party service at request time — the same posture the basemap pipeline already takes
(`bootstrap-basemap.sh` fetches a dated build once, not per request), and it is versioned and
reviewable the way `12-map-architecture.md` already describes regional map state being
delivered: through signed, versioned bundles, not an always-live feed.

This narrows open question 2 below without closing it: authorship for a first `art_register`
is operator-curated (OSM-seeded plus manual), not creator-authored in the existing business
sense. Moderation workflow, review responsibility, and expiry for manual entries remain open.

## Explicitly out of scope for this workstream

- **Transport.** Named separately in `map-data.md`; large enough to need its own workstream and
  its own authority questions once it exists.
- **Autonomy outside SPECTATE.** No background walking, no persistence of intent across a mode
  change.
- **Training-signal egress** from any walk-derived observation.
- **A protocol change implemented from this repository.** The landmark projection is drafted
  and reviewed in `nilx-one/0x1`; this repository only states what it would consume.
- **Expanding the fog itself through Avaia's own movement.** At Stage 1, only the person's own
  device observations light a cell. Whether Avaia's walk should ever do the same is left to a
  later stage, deliberately.

## Open questions

Stated so this document does not silently pick an answer by omission:

1. Per-step admission versus a bounded walk-delegation scope (§2).
2. Moderation workflow, review responsibility, and expiry for `art_register` entries — the
   authorship direction itself (OSM-seeded plus manual) is narrowed above, but the same open
   items `12-map-architecture.md` already lists for creator projections (who moderates, what
   expires, what a rejection looks like) are not resolved by naming the source.
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
8. **MWL8.** At Stage 1, every `DecisionMenu` candidate, and every point a mark may reference,
   is drawn from `ShadeSource.litCells()`; a fogged cell is never constructed as a candidate.
9. **MWL9.** At Stage 1, only a person's own device observations light a cell. An admitted
   `NavigateTo` step must never be the first thing to light the cell it targets.
10. **MWL10.** `art_register` content, OSM-seeded or manual, is product data, not protocol
    truth: it must not be offered to `DecisionMenu`, rendered as map state, or treated as
    `map.registry` content until it has crossed the landmark projection gate (MWL6).

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
