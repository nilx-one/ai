# Choosing the models Avaia loads

Avaia's first slice runs a language model on the owner's device through
[`@aiaiaiai/webllm`](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/docs/browser-local-inference.md).
The foundation pins one default so that its adapter has a tested pair to ship; which models
*this product* serves, and where their artifacts come from, is `ai`'s decision — provider
selection is named as product work in the foundation's own integration contract.

This document is the analysis behind that decision. It is written against what Avaia is
for, and that is the first section because it reorders everything after it.

## What the model is for

Avaia is not a conversational partner and nobody is going to chat with it. It explores,
moves, gradually lives, and chooses where to go next.

What that makes the model's output is a **choice from a closed set the product already
resolved** — this crate's `AvaiaActionProposal`, which is `NavigateTo { target }` over an
opaque `MapTargetId` the world layer minted, or `StopNavigation`. Prose is not the product.
A sentence a person reads is not the product either.

Three consequences, and they are the whole reason this analysis was redone:

- **Language ability stops being a gate.** It was the second-ranked constraint when the
  slice was a Ukrainian-speaking dialogue avatar. A model that never writes a sentence a
  person reads does not need to write Ukrainian sentences.
- **Validity is a decoding property, not a model property.** A grammar-constrained decode
  emits one of the allowed targets or nothing; the model's remaining job is to pick a
  *good* one from a short structured context.
- **The model is resident, not summoned.** Something that lives takes many small decisions
  across a session, so its memory is held rather than spent once, and it has to survive
  losing its own cache.

## What decides it

In order. A candidate that loses a row above never wins on a row below.

1. **Redistribution licence.** Serving weights from our host makes us a distributor of them.
   A licence that permits use but attaches conditions to redistribution attaches those
   conditions to our host.
2. **Judgement over a short structured instruction.** The world state, the allowed targets,
   and what Avaia is trying to do arrive as a small prompt. Following it is the capability
   we are actually buying parameters for.
3. **Resident VRAM.** Held for as long as the runtime is `Active`, alongside everything else
   the product is drawing.
4. **Re-acquisition cost.** A device that lives with Avaia will lose the model to eviction
   and take it back. Those bytes are ours to serve, repeatedly, per device.
5. **Artifacts that exist at the version we pin.** `WebLLM` `0.2.84` resolves a model to a
   prebuilt weight repository *and* a compiled `model_lib` WASM. A model outside that
   registry means compiling `MLC` ourselves, which is a different project.
6. **Bounded, non-thinking generation.** The adapter generates at most 128 new tokens with
   thinking disabled. A reasoning-first checkpoint spends a decision's budget on itself.

## The candidate set

Read off the prebuilt registry of `@mlc-ai/web-llm@0.2.84` — the version `@aiaiaiai/webllm`
pins. VRAM is the registry's own `vram_required_MB` at `q4f16_1`; the `q4f32_1` twins cost
roughly 1.3× and win nothing their half-precision row lost.

| `model_id` | VRAM MB | Licence | Why not |
|---|---:|---|---|
| `SmolLM2-360M-Instruct-q4f16_1-MLC` | 376.06 | Apache-2.0 | The cheapest entry that clears row 1. Rejected on row 2 — see [The floor is a claim](#the-floor-is-a-claim-not-a-fact). |
| `TinyLlama-1.1B-Chat-*`, `RedPajama-*`, `phi-1_5`, `phi-2` | 675–3054 | Apache-2.0 / MIT | Older base and chat checkpoints; instruction-following is the one thing we need and the one thing they are weakest at. |
| `gemma3-1b-it-q4f16_1-MLC` | 711.07 | Gemma Terms | Redistribution carries Google's use policy to whoever receives the weights. |
| `Llama-3.2-1B-Instruct-q4f16_1-MLC` | 879.04 | Llama 3.2 Community | Attribution, a copy of the licence, and the acceptable-use policy travel with every mirror we serve. |
| `gemma-2-2b-it`, `gemma-2b-it` | 1476–1895 | Gemma Terms | As above. |
| `OLMo-2-0425-1B-Instruct-q4f16_1-MLC` | 1776.75 | Apache-2.0 | Clean licence, real candidate, and 373 MB more resident than the floor for the same size class. Cost, not language. |
| `stablelm-2-zephyr-1_6b-q4f16_1-MLC` | 2087.66 | Stability community | Not Apache-2.0; conditions to read before mirroring, for a 1.6B. |
| `Hermes-3-Llama-3.2-3B-q4f16_1-MLC` | 2263.69 | Llama 3.2 Community | Llama licence underneath, at the upgrade entry's budget. |
| `Qwen3.5-0.8B` / `Qwen3.5-2B-q4f16_1-MLC` | 1629.49 / 2245.44 | to confirm | The newer generation, and 226 / 209 MB more resident than the entries we serve. See [Why not the newer generation](#why-not-the-newer-generation). |
| `Ministral-3-3B-Instruct-2512-BF16-q4f16_1-MLC` | 2863.69 | to confirm | 827 MB above the upgrade entry. The `Ministral` line has shipped under Mistral's research licence while `Mistral-7B-Instruct` shipped Apache-2.0, so the family name settles nothing. Its `Base` and `Reasoning` siblings are the wrong checkpoint at the same price. |
| `Phi-3.5-mini-instruct-q4f16_1-MLC` | 3672.07 | MIT | Clean licence; 3672 MB resident, or 2520 MB in the `-1k` variant whose 1024-token window a world-state prompt would consume by itself. |
| `Mistral-7B-Instruct-v0.3-q4f16_1-MLC` | 4573.39 | Apache-2.0 | The cleanest licence in the field at twice the upgrade entry's VRAM. `OpenHermes`, `NeuralHermes`, and `Hermes-2-Pro` on the same base sit at 4033–4573. |
| `DeepSeek-R1-Distill-Qwen-1.5B-*` | — | — | Commented out of the `0.2.84` registry over a correctness issue, so it has no artifacts to serve, and it is a reasoning distillation regardless. |

## The decision: two models, one family

| Role | `model_id` | VRAM MB | Registry says |
|---|---|---:|---|
| Floor — what a mobile WebView can hold while the product draws | `Qwen3-0.6B-q4f16_1-MLC` | 1403.34 | `low_resource_required: true` |
| Upgrade — where a device answers the probe generously | `Qwen3-1.7B-q4f16_1-MLC` | 2036.66 | `low_resource_required: true` |

Two entries because the device decides and one entry cannot be right for both a phone and a
workstation. The same family at both tiers because that means one prompt profile to
validate, one conversation template, one thinking switch, one licence, and one mirror
layout — and because the two sizes are a real gap rather than a rounding difference.

`Qwen3` because it is **Apache-2.0 outright**, which is the only licence in the field that
costs the mirror nothing and the only one that needs no reading before the first byte is
copied. It is instruction-tuned at both sizes, its thinking mode is a switch the adapter
already turns off, and its ABI is a plain paged KV cache — one fewer unknown for an engine
that stays loaded for as long as an owner is spectating.

Selection between the two is not a preference and not a fallback: `select_local_model` in
`src/inference.rs` reads a measured device. A device that runs neither gets `None`, which
means this surface runs no local model. It does not mean offer something smaller, and it
does not mean move inference elsewhere without saying so.

### Why not the newer generation

`Qwen3-0.6B` is the smallest entry in the pinned registry, not the newest: the registry
carries `Qwen3` at 0.6B, 1.7B, 4B, 8B and `Qwen3.5` at 0.8B, 2B, 4B, 9B. When the model
produced prose for a person to read, newer was worth 226 MB. Producing a choice from a
closed set, it is worth the least of everything we are paying for, while resident MB and
re-acquired MB are worth the most — and `Qwen3.5`'s two claims that matter here, Apache-2.0
and its artifacts' provenance, are still unread.

There is also a residency-specific reason. Every `Qwen3.5` entry carries
`max_history_size: 1`, which in `0.2.84` sizes the **RNN state** and is read only for a
model whose resolved ABI needs one — a hybrid holding both a paged KV cache and recurrent
state. That is a second kind of state to reason about in a long-lived resident engine, for
a generation of quality we do not need. If measurement later says the floor picks badly,
`Qwen3.5-0.8B` and `Qwen3.5-2B` are the named step up, and the licence gets read then.

## The floor is a claim, not a fact

`SmolLM2-360M-Instruct-q4f16_1-MLC` is Apache-2.0 and costs **376 MB** — a quarter of the
floor we chose, and cheaper still to take back after eviction. With language out of the
ordering, the only thing standing between it and the floor is row 2: whether a 360M model
follows a short world-state instruction well enough to choose a *good* target, given that a
grammar already guarantees it chooses a *valid* one.

Our claim is that it does not, and that below roughly 0.6B a constrained decode produces
motion rather than life. That is a claim about behaviour, and behaviour is measurable:
replay a fixed set of world states through both models under the same grammar and compare
the choices. If the small one holds up, the floor moves down and the mirror gets four times
cheaper. It is the first experiment worth running after the probe.

## Living on the device

"It lives on the device, and if it fell out of cache it takes it back" is a lifecycle, and
the pinned stack supplies only half of it.

**Nothing here asks for persistence.** `WebLLM` never calls `navigator.storage.persist()`,
so model artifacts sit in ordinary script-writable storage, which WebKit in particular
evicts after a stretch without interaction. Eviction is therefore the default expectation,
not an edge case. Asking for persistence is the product's call to make, and the answer may
be refused — a refusal is a fact to read and render, never one to assume either way.

**Taking it back is already a modelled state, and it must stay explicit.** `probe()` never
downloads; it reports `supported(cached: false)` for exactly this situation, and `load()`
is the only call allowed to fetch. The foundation is explicit that a product must present
that download honestly and must not call `load()` as a side effect of probing. So a
re-acquisition is a visible act — the same one as the first install, with the same bytes
from the same immutable revision — and not a stall an owner cannot account for.

**Residency maps onto the modes this crate already has.** `Spectate` is `Active`: the
engine stays loaded and decisions are cheap. `Manual` is `Quiescing`: generation is
interrupted at a boundary and GPU resources may be released. `Offline` is `Dormant`:
unloaded, while `unload()` deliberately keeps the downloaded cache — losing the weights is
eviction's doing, never ours.

**The cache backend is a choice we have not made yet.** `0.2.84` supports `cache`,
`indexeddb`, `cross-origin`, and `opfs`, with the Cache API the best-tested and the default.
`opfs` is the one worth measuring for something meant to stay: different backends are
evicted under different pressure. Whatever we choose has to be chosen *through the seam
below*, because the adapter passes no `AppConfig` to `hasModelInCache` — a mirrored model
list plus a non-default backend means cache lookups that answer about the wrong place.

## Choosing rather than talking

The single most load-bearing gap between this pin and what Avaia needs is not the model. It
is constrained decoding.

`WebLLM` `0.2.84` supports `response_format` with `json_object` (a schema), `grammar` (EBNF),
and `structural_tag`. The foundation adapter's `stream()` passes messages, `max_tokens`,
`temperature`, `top_p`, and `enable_thinking: false` — and no `response_format`. Without it,
what comes back is free-form text that the product would have to parse into a `MapTargetId`,
which is precisely the ambiguity the closed action set exists to remove.

With it, the grammar is generated from the targets the world layer already resolved, so the
decode cannot name a target that does not exist, and this crate's refusal — that a model may
not mint a `MapTargetId` — is enforced at the point of generation as well as after it. A
model output is still a proposal, still admitted or refused at the authority boundary; the
grammar only removes the class of proposals that are meaningless.

Closing the gap is the same seam as pointing the loader at our host, and the next section
is where that seam is.

## Not measured yet

The foundation's [WebGPU probe](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/probes/webgpu/RESULTS.md)
has four surface rows and every one reads `not measured`. Mobile Safari, the messaging
mini-app WebView, and the sandboxed activity iframe are the surfaces this product is opened
on, and none of them has answered. So the split above is a shortlist conclusion, not a
device fact, and three results would move it:

- **No `shader-f16` on a surface.** No `q4f16_1` entry runs there, and `select_local_model`
  already returns `None` for such a device. Per the probe's own reading, that surface gets a
  deterministic product path — not a `q4f32_1` fallback it degrades into, which would cost
  1925 MB on the weakest device we have.
- **A `maxStorageBufferBindingSize` below what the weights need.** A loading constraint: the
  artifacts get chunked to that ceiling. It does not change which models we chose.
- **No adapter at all.** That surface never renders a local model as available, and Avaia is
  remote or absent there.

Running the probe on the three embedded surfaces is the gate on committing to a mirror.
Mirroring is bytes we pay for; measuring is a page someone opens.

## Serving the artifacts from our host

**What the pinned registry points at today.** Weights resolve to `huggingface.co`, and
`model_lib` resolves to
`raw.githubusercontent.com/mlc-ai/binary-mlc-llm-libs/main/web-llm-models/v0_2_84/base/…`.
That path names a *branch*, so the WASM our runtime executes is not pinned by our pin — it is
whatever `main` holds at load time, fetched from a host that is not a CDN and that sees the
IP of every device Avaia lives on, every time eviction sends it back for the weights.

**What has to be mirrored per model.** `mlc-chat-config.json`, `tokenizer.json`,
`ndarray-cache.json`, every `params_shard_*.bin`, and the `model_lib` WASM — twice, once per
served entry.

**The URL shape is not free.** `WebLLM` normalizes `ModelRecord.model` by appending
`resolve/main/` unless the URL already matches `…/resolve/…/`. Serving from
`…/<model>/resolve/<revision>/` satisfies the normalizer and gives us an immutable revision
segment: cacheable forever, rolled forward explicitly, and identical bytes on every
re-acquisition.

**Integrity covers three of the five artifacts.** `ModelRecord.integrity` verifies the
config, the `model_lib`, and the tokenizer files against SRI hashes. The weight shards are
not covered. Pinning shard bytes is ours to do — the immutable revision segment plus the
hashes recorded where we record the mirror.

**We declare the feature gate the registry omits.** The `Qwen3`, `Qwen3.5`, `Llama-3.2`,
`gemma3-1b`, and `Ministral-3` entries carry no `required_features`, while the older
`q4f16_1` entries carry `["shader-f16"]`. A device without f16 therefore downloads a whole
model before failing to initialize it. Our served `model_list` declares
`required_features: ["shader-f16"]` on every entry — which is why the catalog derives that
flag from the quantization rather than storing an opinion about it.

## Where the choice lives in code

`src/inference.rs` holds the catalog and the selection rule: the served `model_id`s, the
quantization, the VRAM figure and context window each was measured at, the derived
`shader-f16` requirement, and `select_local_model` reading a `DeviceCapability` that was
measured rather than assumed. It downloads nothing and probes nothing.

Everything else needs one seam that does not exist yet. `LocalInferenceRuntime` takes a
model id; `WebLlmBrowserHost` takes a worker factory. Neither accepts an `AppConfig`, so
neither can be pointed at our origin, given a cache backend, or handed integrity hashes —
and `stream()` accepts no `response_format`, so neither can be asked for a constrained
decode. Two ways to close it:

1. **`nilx-one/web` implements `LocalInferenceHost` itself.** The interface is exported and
   the runtime takes it by injection, so a product host can pass its own `appConfig` to
   `CreateWebWorkerMLCEngine` and its own `response_format` to the completion. Interim, and
   no foundation change.
2. **The adapter grows both seams upstream.** Product-agnostic, and the right home: every
   product that mirrors artifacts needs the first, and every product whose model chooses
   rather than talks needs the second.

Either way the model identity — which `model_id`, which revision, which quantization — and
the shape of an admissible decision are `ai`'s to state and `web`'s to consume.

## Related

- [Browser-local inference](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/docs/browser-local-inference.md)
- [WebGPU capability probe](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/probes/webgpu/README.md)
- [Integrating a product AI runtime](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/docs/nilx-one-ai-integration.md)

---

© 2026 aiaiaiai · aiaiaiai.org
