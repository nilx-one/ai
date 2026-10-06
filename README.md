# ai

The product-specific artificial intelligence runtime for 0x1.

Protocol truth and deterministic shared product behavior remain owned by the canonical `nilx-one/0x1` specification and `nilx-one/core`. Where Avaia walks is not protocol truth, and it is hers: she decides, and she goes.

```text
avaia decided  -> avaia went; nothing admits or approves a step
owner          -> influences her only by setting a navigation position
a walk         != presence, interaction or BondChain fact
model output   != protocol truth; simulation != BondChain fact
```

## Local inference

Avaia's model chooses where to go next, not what to say. [Choosing the models Avaia
loads](docs/model-selection.md) states which models this runtime serves, the constraints
that decided them, what living on a device costs when its cache is evicted, what is still
unmeasured, and what serving the artifacts from our own host requires. The catalog and the
selection rule are `src/inference.rs`, and the closed action set a decode may not step
outside is `src/decision.rs`.

## Evolving Avaia

Avaia is not a chatbot personalized around a conversation history. Its long-term personalization is shaped primarily by observed behavior and interaction evidence: choices, repeated preferences, rejections, recurrence, context-sensitive patterns, and other permitted individual signals.

The intended direction is cumulative:

```text
observed interactions
        ↓
behavioral evidence
        ↓
preferences and patterns
        ↓
stable tendencies
        ↓
personal traits
        ↓
Avaia identity
```

The upper layers are increasingly derived and increasingly slow to change. A single action may be evidence; repeated behavior across time and context may support a preference or tendency; only durable, cross-context consistency should influence higher-level traits. Avaia identity is the long-lived result of that evolution, not a personality preset and not a mutable profile field.

Explicit self-description can be useful evidence, but it is not authoritative over observed history. Likewise, a learned inference must never be promoted into a BondChain fact merely because the model is confident in it.

```text
BondChain               = observed interaction history
behavioral evidence     = permitted signals derived from or associated with experience
preferences / patterns  = learned, revisable interpretations
traits                   = slower, higher-order derived structure
Avaia identity          = persistent personal intelligence state shaped over time
```

The model-specific internal representation of that state may be latent rather than a flat list of labels. Human-readable traits and preferences are projections of the learned state, not necessarily the state itself.

Two Avaia instances beginning from the same base model should be able to diverge substantially as their Bonds accumulate different histories. Updating the base model must not erase that personal evolution. Conversely, running different model sizes or quantizations on different devices must not create separate identities; those are runtime representations of the same Bond-level personal state.

The hard boundary remains:

> Avaia may learn from the Chain, but may never rewrite the Chain to fit what it learned.

[One mind, many models](docs/mind.md) states what may travel between a person's devices
when those devices run different models, different sizes of model, or none at all — and the
line that decides it: if a value's meaning depends on which model produced it, it is a
cache, not state.

[What belongs upstream, and what does not](docs/foundation-gaps.md) records the seams this
slice found on the wrong side of the foundation boundary, and the vocabulary that must stay
on this one. [The implementation plan](docs/implementation-plan.yaml) is the same work
ordered, with what blocks what; the issues remain the source of truth.

[Avaia walks the map, only while spectating](docs/map-walk-and-landmarks.md) scopes a walk as
a chain of the existing NavigateTo proposal, states what a landmark projection would need from
`nilx-one/0x1` before there is anything to discover, and names what it deliberately excludes —
transport, and any autonomy outside SPECTATE.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md), [CLA.md](CLA.md), and [TRADEMARKS.md](TRADEMARKS.md) before submitting substantial work.

New authored source and configuration files must carry the canonical aiaiaiai copyright signature and `SPDX-License-Identifier: MPL-2.0` when the format supports comments. Repository policy CI validates this automatically.

## License

Licensed under the Mozilla Public License, Version 2.0 (`MPL-2.0`). See [LICENSE](LICENSE) and [NOTICE](NOTICE).

---

© 2026 aiaiaiai · aiaiaiai.org
