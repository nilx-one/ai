# Describing a landmark

**Status:** implemented here as a faculty (`src/describe.rs`, `fixtures/landmark-description.json`).
When a landmark is described is the drive's, in [`nilx-one/core`](https://github.com/nilx-one/core)
(`docs/avaia-drive.md`). How it is shown and kept is the host's. This document states the whole
loop and the part this repository owns.

## The loop

```text
host world layer        Core drive                 host                    ai faculty
passing{ref, described} ──▶ is it new? ──describe{at}──▶ facts of `at` ──▶ template (tier 0)
                                                        │                  retelling (tier 1+)
                         ◀──described{at}────────────── │ ◀── admit() ─────┘
                         show{at, as, ms} ─────────────▶ shows it on every surface
                         keep{at} ─────────────────────▶ notebook: the short description
```

1. **Core sees what to describe.** The host reports what a walk passes, and whether its
   notebook already holds a description of it (`described`). When the Avaia stops in front of a
   landmark that is not described, and not described already this session, the drive asks the
   host for one (`describe`). The drive never sees the text, the name or the place: it decides
   *when*, and *how* it is shown.
2. **The host asks this faculty.** It sends the landmark's archive facts
   ([`LandmarkFacts`](../src/describe.rs)), never anything about the person.
3. **The faculty describes.**
   - Tier 0 is a deterministic description worded from a closed set of facts (name, kind,
     year, author, architect, height, material). It is the product. Without a model, this is
     what she says and what is kept.
   - With a local model, the model is asked to retell that description in her own words, and
     [`admit`](../src/describe.rs) decides whether the retelling may stand. It is refused if
     it is too long, has markup, is not finished as at most three sentences, has a number the
     description does not have, or leaves out the landmark's name. A refusal is no answer: the
     tier-0 description stands.
   - The model runs on the device, as every faculty here does. On a host that cannot run one
     (a Telegram or Discord webview without WebGPU, a watch), tier 0 is the whole answer.
4. **Core says what to show and how.** The host answers `described`. The drive then emits
   `show` (which landmark, as what, for how long), and in the same answer `keep`. Every host
   (web, Telegram, Discord) carries out the same commands.
5. **The host keeps a short description.** It is written into the device-local
   landmark notebook beside the `seen` or `studied` entry. Per [`mind.md`](mind.md) it travels
   as an envelope: the text, and what produced it (`template` with its version, or the model
   id). A retelling is a cache: a device may throw it away and word tier 0 again. The tier-0
   text is the landmark's facts, and means the same everywhere.

## What the prompt may carry

Only the landmark's archive facts from the closed set, each a short single line of plain text.
A value that is long, multi-line or looks like markup is dropped, not repaired, so an
archive's free text (an inscription, a note, a link) never reaches a model. Never the person's
position, journal, identity or notebook.

## Invariants

1. **LD1.** A description says only what the archive declares. A model retells it and never
   adds to it; a retelling that may carry a new fact is discarded, never corrected.
2. **LD2.** The prompt carries the closed set of a landmark's facts and nothing about the
   person.
3. **LD3.** Tier 0 is the product. Every host can describe every landmark without a model.
4. **LD4.** The drive decides when and how, and never sees the text. The host words, shows
   and keeps it.
5. **LD5.** A kept description is device-local notebook content (MWL4, MWL5): never synced,
   exported or used as training signal, and never presence, attendance or BondChain.

---

© 2026 aiaiaiai · aiaiaiai.org
