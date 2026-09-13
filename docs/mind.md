# One mind, many models

The mind is ours and the model is a faculty it happens to have on this device today. A
person carries one Avaia across a workstation, a phone and a watch; those three machines
will not run the same model, will not run the same *size* of model, and on the smallest of
them may run none at all. Nothing about who Avaia is may depend on which of them is in hand.

This document states what that costs: what may travel between devices, what may not, what a
device declares about itself, and how two devices that disagree are reconciled.

## The line

> If a value's meaning depends on which model produced it, it is a cache, not state.

Everything below follows from that sentence.

**State** is product vocabulary: what was decided, what was admitted, what a person set,
what Avaia has been asked to do. It is meaningful without a model in the room, it is what
syncs, and it is the only thing that is authoritative.

**Cache** is everything a model produced or that only a model can interpret: rephrased
sentences, embeddings, KV state, logits, scores, prompt texts. It is recomputable, it is
per-device, and a device is always free to throw it away. Embeddings are the trap worth
naming: they are tempting to sync as "memory", and they are tokenizer- and model-specific,
so a vector written by a 1.7B model is noise to a 0.4B one. They stay local, derived, and
disposable.

**Faculties** are what this device can do *right now*. They are discovered, never assumed,
never synced, and never part of identity.

## Model identity is not subject identity

The foundation already refuses to let a model become a person: `RuntimeId`, `ModelId` and
`ControllerId` are replaceable by construction, and replacing a runtime may not mint a
different subject. So heterogeneous devices are legal by construction — what is missing is
not permission but a state contract, which is what this document is for.

## The capability ladder

A faculty is named by the **task** it serves, never by the model that serves it. Each task
has a tier-0 implementation that is deterministic, always available, and *is the product* —
not a fallback a richer tier degrades into.

| Tier | Roughly | Serves |
|---|---|---|
| 0 | no model at all | the whole product, deterministically |
| 1 | featherweight, ≲0.5 GB, a watch or a weak phone | constrained rephrasing, classification into a closed set |
| 2 | small, ~1–2 GB, a phone or better | choosing among offered options with short reasoning |
| 3 | anything larger | not this product's policy today; inference is on-device |

Two rules keep this from becoming a quality lottery:

1. **Same kind, different quality.** A tier changes how something reads or how well it is
   chosen. It never changes *what* the product does, what it is allowed to do, or what a
   result means. A watch runs a small product, not a broken one.
2. **Tier is declared by the device, not by the account.** A device answers with what it
   measured — `nilx-one/web` already does this for narration: a probe, a capability result,
   and a refusal that names its reason.

## What syncs

An append-only log per device. Each entry carries the device that wrote it, a sequence
number within that device, a hybrid logical clock, and a payload in product vocabulary.
State is the fold over the merged logs; merging is a union, so two devices that both went
offline and both wrote converge without a server deciding anything.

The server stores opaque blobs. The mind is personal, so it travels encrypted end to end and
the store learns sizes and timing and nothing else. No merge logic lives there — a server
that could merge would be a server that could read.

## What a derivation carries when it does travel

Sometimes a derived value is worth keeping: an accepted phrasing, a choice a person approved.
It travels as an envelope, never bare:

```text
value                      what it is, in product vocabulary
source                     the evidence it was derived from
produced_by { faculty, adapter, model_id, model_revision, template_version }
admitted                   whether authority admitted it, or it is only a cache
```

Merging two envelopes for the same slot:

- an **admitted** value beats a cache, whatever produced it;
- between caches, the **higher declared tier** wins, because a device that could do better
  did;
- ties break by hybrid clock;
- a device **never** overwrites a higher-tier derivation with a lower-tier one. The watch
  does not flatten what the workstation wrote.

## Schema, across versions that do not match

Devices update at different times, so a mind will routinely be read by an older client than
the one that wrote it. Two rules:

- **Unknown fields are preserved verbatim.** A client that re-serialises an entry it only
  partly understands must write back what it read, or an old phone silently deletes what a
  new one knows.
- **An unreadable major version stops writes, not reads.** A client that cannot interpret a
  newer schema keeps working locally and refuses to publish into the shared log, rather than
  clobbering it with a downgraded view.

## The conflict this design must not paper over

`nilx-one/web`'s `presence-contract` says the presence journal is personal, device-local and
append-only, with **no sync or network operation in its contract** — deliberately. So:

- **The journal does not sync.** Not as a default, not as a convenience.
- The mind must be *useful without it*: what Avaia carries between decisions is not the raw
  rows of where a person stood.
- Narration derived from the journal is journal-derived data. If a derived sentence travels
  to another device, that is journal egress, and it is governed by the egress policy
  `nilx-one/ai#6` is meant to define — not by this document, and not as a side effect of the
  word "sync".

The honest position for the first version: **the mind syncs, the journal does not, and
journal-derived caches stay on the device that derived them.** Lifting that is a separate,
deliberate decision with its own privacy contract.

## What this means for the model choice already made

Nothing about `docs/model-selection.md` changes, and two of its conclusions get stronger.
The catalog exists per device rather than per account, which is what `select_local_model`
already assumes. And the deterministic path stops being a concession for weak surfaces: it
is tier 0, the floor the whole ladder stands on, and the reason a device with no model at
all is still a device this product runs on.

## Open, and not decided here

- Which slots a mind actually has. This is `nilx-one/ai#15`, and it is a product question
  before it is a data-structure one.
- Where the encrypted log lives and what authenticates a device into it. Storage is this
  repository's trust boundary; the identity service owns who a person is.
- Whether a device may *request* a derivation from a better-equipped device it shares an
  account with. Tempting, and it makes one device a provider for another — which is remote
  inference wearing a friendly hat, and therefore a policy change rather than a feature.

## Related

- [Choosing the models Avaia loads](model-selection.md)
- [What belongs upstream, and what does not](foundation-gaps.md)
- [The implementation plan](implementation-plan.yaml)

---

© 2026 aiaiaiai · aiaiaiai.org
