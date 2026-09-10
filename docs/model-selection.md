# Choosing the models Avaia loads

Avaia's first slice runs a language model in the browser through
[`@aiaiaiai/webllm`](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/docs/browser-local-inference.md).
The foundation pins one default so that its adapter has a tested pair to ship; which models
*this product* serves, and where their artifacts come from, is `ai`'s decision — provider
selection is named as product work in the foundation's own integration contract.

This document is the analysis behind that decision. The decision is two models, and it is
provisional in one specific way stated in [Not measured yet](#not-measured-yet).

## What decides it

In order. A candidate that loses a row above never wins on a row below.

1. **Redistribution licence.** Serving weights from our host makes us a distributor of them.
   A licence that permits use but attaches conditions to redistribution attaches those
   conditions to our host.
2. **Ukrainian.** Avaia answers the person in their language. A model whose card does not
   claim Ukrainian is not a model we can put in front of a Ukrainian-speaking owner.
3. **The surface the product is actually opened on.** A mobile WebView, not a workstation.
   The VRAM ceiling and `shader-f16` decide this, and both are device facts.
4. **First-load cost.** Every first load is bytes we serve. It is also the wait a person
   sits through before Avaia says anything.
5. **Artifacts that exist at the version we pin.** `WebLLM` `0.2.84` resolves a model to a
   prebuilt weight repository *and* a compiled `model_lib` WASM. A model outside that
   registry means compiling `MLC` ourselves, which is a different project.
6. **Bounded, non-thinking generation.** The adapter generates at most 128 new tokens by
   default with thinking disabled. A reasoning-first checkpoint spends that budget on
   itself.

## The candidate set

Read off the prebuilt registry of `@mlc-ai/web-llm@0.2.84` — the version `@aiaiaiai/webllm`
pins. `vram_required_MB` and the context override are the registry's own values.

| `model_id` | VRAM MB | low-resource | ctx | Family licence | Ukrainian claimed |
|---|---:|---|---:|---|---|
| `gemma3-1b-it-q4f16_1-MLC` | 711.07 | yes | 4096 | Gemma Terms | 1B is the English-first variant |
| `Llama-3.2-1B-Instruct-q4f16_1-MLC` | 879.04 | yes | 4096 | Llama 3.2 Community | no — 8 official languages |
| `Qwen3-0.6B-q4f16_1-MLC` | 1403.34 | yes | 4096 | Apache-2.0 | yes |
| `Qwen3.5-0.8B-q4f16_1-MLC` | 1629.49 | yes | 4096 | to confirm | to confirm |
| `SmolLM2-1.7B-Instruct-q4f16_1-MLC` | 1774.19 | yes | 4096 | Apache-2.0 | no — English |
| `gemma-2-2b-it-q4f16_1-MLC` | 1895.30 | no | 4096 | Gemma Terms | not stated for this size |
| `Qwen3-1.7B-q4f16_1-MLC` | 2036.66 | yes | 4096 | Apache-2.0 | yes |
| `Qwen3.5-2B-q4f16_1-MLC` | 2245.44 | no | 4096 | to confirm | to confirm |
| `Llama-3.2-3B-Instruct-q4f16_1-MLC` | 2263.69 | yes | 4096 | Llama 3.2 Community | no — 8 official languages |
| `Ministral-3-3B-Instruct-2512-BF16-q4f16_1-MLC` | 2863.69 | yes | 4096 | to confirm | to confirm |

`q4f32_1` twins exist for every row at roughly 1.3× the VRAM. They are the shape a device
without `shader-f16` would need, and [Not measured yet](#not-measured-yet) says why that is
not the same as a fallback.

**`Qwen3-0.6B` is the smallest entry in the registry, not the newest.** The pinned registry
carries `Qwen3` at 0.6B, 1.7B, 4B, and 8B, and `Qwen3.5` at 0.8B, 2B, 4B, and 9B. Only the
first two sizes of each are within reach of the surfaces this product opens on.

The licence and language columns are the two this repository cannot settle by reading code,
and `to confirm` means exactly that — see [Before a byte is mirrored](#before-a-byte-is-mirrored).

## What the ordering eliminates

**English-only families, at any size.** SmolLM2, TinyLlama, `phi-1_5`, `phi-2`, RedPajama,
and Gemma's 1B: the cheapest rows in the registry, and none of them is a model Avaia can
answer a Ukrainian owner with. Size never buys back a language a model was not trained on.

**Llama 3.2, at both sizes.** Ukrainian is not among its eight official languages, so it
loses on row 2 before its licence is reached. That licence would have cost us the
`Built with Llama` attribution, a copy of the community licence, and the acceptable-use
policy travelling with every mirror we serve.

**Gemma's terms, for the sizes that could have qualified.** Redistribution carries Google's
use policy to whoever receives the weights. That is a term we would be passing on from our
own domain, and 2B does not buy enough to be worth taking it on.

**Reasoning-first checkpoints.** `Ministral-3-3B-Reasoning`, and any Qwen variant run with
thinking enabled: the adapter disables thinking and caps generation at 128 tokens, so a
model that reasons before answering answers less within the same budget.

## The decision: two models, one family

| Role | `model_id` | VRAM MB | Registry says |
|---|---|---:|---|
| Floor — the entry a phone can plausibly grant | `Qwen3.5-0.8B-q4f16_1-MLC` | 1629.49 | `low_resource_required: true` |
| Upgrade — where a device answers the probe generously | `Qwen3.5-2B-q4f16_1-MLC` | 2245.44 | `low_resource_required: false` |

Two entries rather than one because the device decides, and because the alternative to a
second entry is not simplicity — it is either a phone-sized model on a workstation or a
model no phone can load. Two entries of the *same family* because one family means one
conversation template, one thinking switch, one system-prompt profile, one licence
question, and one mirror layout.

`Qwen3.5` rather than `Qwen3` because it is the newer generation at both of the sizes we
can afford, and because `Qwen3.5-0.8B` costs 226 MB more than `Qwen3-0.6B` for a whole
generation of difference. `q4f16_1` at both sizes because the half-precision shape is the
only one a phone has the memory for: the full-precision twin of the smaller entry (1894.19)
costs more than the larger entry saves.

Selection between the two is not a preference and not a fallback — it is
`select_local_model` in `src/inference.rs` reading a measured device. A device that runs
neither gets `None`, which means this surface runs no local model. It does not mean offer
something smaller, and it does not mean move inference elsewhere without saying so.

### What `Qwen3.5` brings with it

Every `Qwen3.5` entry in the pinned registry carries `max_history_size: 1`. In `0.2.84`
that value sizes the **RNN state**, and it is read only for a model whose resolved ABI
needs one — a hybrid architecture holding both a paged KV cache and recurrent state. Two
things follow. The pin matters more than usual, because that hybrid ABI resolution is what
makes these entries loadable at all. And the value is an allocation, not a conversation
limit: `1` is the registry's browser-safe default and the configuration its VRAM figure was
measured under, so raising it is a separate decision that re-opens the number. Bounded
dialogue history stays what it already was — the product's, inside the 4096-token window.

## What this decision does not claim

A 0.8B model's Ukrainian is thin. It will hold a short exchange and it will not hold a long
one. This is acceptable for the first slice for one structural reason: what it produces is
a *proposal*, bounded to 128 tokens, wrapped by `ai` and admitted or refused at the
authority boundary. It is never an authorized message and never a completion.

If the product later needs dialogue quality this size cannot reach, the answer is a declared
remote provider with its own reported transition — the foundation is explicit that a
provider change is reported, not substituted. It is not a bigger local model on a phone, and
it is not a quiet fallback.

## Before a byte is mirrored

Two claims about `Qwen3.5` are load-bearing here and neither has been verified against the
upstream model card: that it is **Apache-2.0**, and that it **claims Ukrainian**. Mirroring
weights is redistribution, so the licence has to be read before the first byte is copied,
not after.

If either claim fails, the same two roles are filled by `Qwen3-0.6B-q4f16_1-MLC` (1403.34)
and `Qwen3-1.7B-q4f16_1-MLC` (2036.66): same vendor, Apache-2.0, Ukrainian among the
claimed languages, one generation older, and two string-and-number changes in
`src/inference.rs`. The structure of the decision does not move; only its two entries do.

## Not measured yet

The foundation's [WebGPU probe](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/probes/webgpu/RESULTS.md)
has four surface rows and every one of them reads `not measured`. Mobile Safari, the
messaging mini-app WebView, and the sandboxed activity iframe are the surfaces this product
is opened on, and none of them has answered.

So the split above is a shortlist conclusion, not a device fact, and three results would
move it:

- **No `shader-f16` on a surface.** No `q4f16_1` entry runs there, and `select_local_model`
  already returns `None` for such a device. Per the probe's own reading, that surface gets
  a deterministic product path — not a `q4f32_1` fallback it degrades into, which would
  cost 1894 MB on the weakest device we have.
- **A `maxStorageBufferBindingSize` below what the weights need.** A loading constraint: the
  artifacts get chunked to that ceiling. It does not change which models we chose.
- **No adapter at all.** That surface never renders a local model as available, and Avaia is
  remote or absent there.

Running the probe on the three embedded surfaces is the gate on committing to a mirror.
Mirroring is bytes we pay for; measuring is a page someone opens.

## Serving the artifacts from our host

Independent of which models win, self-hosting is worth doing on its own terms.

**What the pinned registry points at today.** Weights resolve to `huggingface.co`, and
`model_lib` resolves to
`raw.githubusercontent.com/mlc-ai/binary-mlc-llm-libs/main/web-llm-models/v0_2_84/base/…`.
That path names a *branch*. The WASM our runtime executes is therefore not pinned by our
pin, it is whatever `main` holds at load time, fetched from a host that is not a CDN and
that sees the IP of every person who opens the product. Mirroring makes the pin real.

**What has to be mirrored per model.** `mlc-chat-config.json`, `tokenizer.json`,
`ndarray-cache.json`, every `params_shard_*.bin`, and the `model_lib` WASM — twice, once per
served entry.

**The URL shape is not free.** `WebLLM` normalizes `ModelRecord.model` by appending
`resolve/main/` unless the URL already matches `…/resolve/…/`. A mirror served from a plain
path silently acquires that segment. Serving from `…/<model>/resolve/<revision>/` satisfies
the normalizer and gives us an immutable revision segment to cache forever and to roll
forward explicitly.

**Integrity covers three of the five artifacts.** `ModelRecord.integrity` verifies the
config, the `model_lib`, and the tokenizer files against SRI hashes. The weight shards are
not covered by it. Pinning shard bytes is therefore ours to do — an immutable revision
segment plus the hashes recorded where we record the mirror.

**We declare the feature gate the registry omits.** The `Qwen3`, `Qwen3.5`, `Llama-3.2`,
`gemma3-1b`, and `Ministral-3` entries carry no `required_features`, while the older
`q4f16_1` entries carry `["shader-f16"]`. A device without f16 therefore downloads a whole
model before failing to initialize it. Our served `model_list` declares
`required_features: ["shader-f16"]` on every `q4f16_1` entry — which is why the catalog
derives that flag from the quantization rather than storing an opinion about it.

**Two more things the mirror buys.** Cross-origin isolation and cache headers become ours to
set, and a first load stops being a third-party fetch from our owners' devices.

## Where the choice lives in code

`src/inference.rs` holds the catalog and the selection rule: the served `model_id`s, the
quantization, the VRAM figure and context window each was measured at, the derived
`shader-f16` requirement, and `select_local_model` reading a `DeviceCapability` that was
measured rather than assumed. It downloads nothing and probes nothing; it answers which
model a device may be offered, or none.

The host still has to point `WebLLM` at our origin, and neither `LocalInferenceRuntime` nor
`WebLlmBrowserHost` accepts an `AppConfig` today. Two ways to close that:

1. **`nilx-one/web` implements `LocalInferenceHost` itself.** The interface is exported and
   the runtime takes it by injection, so a product host can pass its own `appConfig` to
   `CreateWebWorkerMLCEngine`. Interim, and no foundation change.
2. **`WebLlmBrowserHost` accepts an optional `AppConfig`.** Upstream, product-agnostic, and
   the right home for it: every product that mirrors artifacts needs the same seam.

Either way the model identity — which `model_id`, which revision, which quantization — is
`ai`'s to state and `web`'s to consume, alongside the system prompt and bounded history `ai`
already owns.

## Related

- [Browser-local inference](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/docs/browser-local-inference.md)
- [WebGPU capability probe](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/probes/webgpu/README.md)
- [Integrating a product AI runtime](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/docs/nilx-one-ai-integration.md)

---

© 2026 aiaiaiai · aiaiaiai.org
