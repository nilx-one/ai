# Describing a landmark

**Status:** implemented here as a faculty (`src/describe.rs`, `fixtures/landmark-description.json`).
When a landmark is described is the drive's, in [`nilx-one/core`](https://github.com/nilx-one/core)
(`docs/avaia-drive.md`). How it is shown and kept is the host's. This document states the whole
loop and the part this repository owns.

## The loop

Core owns the request and result envelope. `src/describe.rs::execute_description`
is the faculty entry point: given Core's closed archive-fact request and an optional
local model proposal with model id, it returns `{text,produced_by,template_version}`.
A denied model proposal returns tier 0; the host does not invent text or acknowledge
completion without a result.

```text
host world layer → passing{ref, described:false} → Core drive: describe{at}
host world layer → description_facts{at, request} → Core validates facts
Core drive → ai_describe{at, request_id, request} → AI faculty on-device
AI faculty → ai_described{at, request_id, result} → Core checks answer
Core drive → show{at, how, ms, result} + keep{at, result} → host Web/iOS/MiniApp
```

1. **Core detects the landmark.** The host world adapter reports opaque refs and notebook status; Core decides when to request facts.
2. **Core builds the AI contract.** The adapter returns `description_facts` with only permitted archive fields. Core validates them and emits `ai_describe` with a request id. The host may transport it but cannot substitute a prompt.
3. **AI describes.** `execute_description` produces deterministic tier 0, or an admitted local-model retelling. The output is a typed result rather than a success flag. When the local model is unavailable the faculty itself returns tier 0.
4. **AI returns to Core.** The `ai_described` input binds the result to the exact request id, landmark and deadline. Core rejects late, mismatched, malformed and cancelled responses and alone emits `show` plus `keep` containing the accepted text.
5. **Host renders and keeps.** Web, iOS and Mini Apps display Core's result and store it in their local landmark notebook. No host may independently declare a description accepted.

`nilx-one/web` currently has an adapter port of the faculty for on-device execution. It must stay fixture-compatible with this repository until both hosts can consume the shared compiled AI faculty directly. The adapter is a runtime implementation detail, not a second authority over generation.

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
4. **LD4.** Core decides when and how, prepares the AI task, receives and checks the
   result. AI owns wording and admission; the host transports, shows and keeps only Core's commands.
5. **LD5.** A kept description is device-local notebook content (MWL4, MWL5): never synced,
   exported or used as training signal, and never presence, attendance or BondChain.

---

© 2026 aiaiaiai · aiaiaiai.org
