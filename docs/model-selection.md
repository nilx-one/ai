# Choosing the model Avaia loads

Avaia's first slice runs a language model in the browser through
[`@aiaiaiai/webllm`](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/docs/browser-local-inference.md).
The foundation pins a default so that its adapter has a tested pair to ship; which model
*this product* loads, and where the artifacts come from, is `ai`'s decision — provider
selection is named as product work in the foundation's own integration contract.

This document is the analysis behind that decision, and the decision is provisional in one
specific way stated in [Not measured yet](#not-measured-yet).

## What decides it

In order. A candidate that loses a row above never wins on a row below.

1. **Redistribution license.** Serving weights from our host makes us a distributor of them.
   A license that permits use but attaches conditions to redistribution attaches those
   conditions to our host.
2. **Ukrainian.** Avaia answers the person in their language. A model whose card does not
   claim Ukrainian is not a model we can put in front of a Ukrainian-speaking owner.
3. **The surface the product is actually opened on.** A mobile WebView, not a workstation.
   VRAM ceiling and `shader-f16` availability decide this, and both are device facts.
4. **First-load cost.** Every first load is bytes we serve. It is also the wait a person
   sits through before Avaia says anything.
5. **Artifacts that exist at the version we pin.** WebLLM `0.2.84` resolves a model to a
   prebuilt weight repository *and* a compiled `model_lib` WASM. A model outside that
   registry means compiling MLC ourselves, which is a different project.
6. **Bounded, non-thinking generation.** The adapter generates at most 128 new tokens by
   default with thinking disabled. A reasoning-first model spends that budget on itself.

## The candidate set

Read off the prebuilt registry of `@mlc-ai/web-llm@0.2.84` — the version
`@aiaiaiai/webllm` pins. `vram_required_MB` and the context override are the registry's own
values, not estimates.

| `model_id` | VRAM MB | low-resource | ctx | Family license | Ukrainian claimed |
|---|---:|---|---:|---|---|
| `gemma3-1b-it-q4f16_1-MLC` | 711.07 | yes | 4096 | Gemma Terms | 1B is the English-first variant |
| `Llama-3.2-1B-Instruct-q4f16_1-MLC` | 879.04 | yes | 4096 | Llama 3.2 Community | no — 8 official languages |
| `Qwen3-0.6B-q4f16_1-MLC` | 1403.34 | yes | 4096 | Apache-2.0 | yes |
| `Qwen3.5-0.8B-q4f16_1-MLC` | 1629.49 | yes | 4096 | unverified | unverified |
| `SmolLM2-1.7B-Instruct-q4f16_1-MLC` | 1774.19 | yes | 4096 | Apache-2.0 | no — English |
| `gemma-2-2b-it-q4f16_1-MLC` | 1895.30 | no | 4096 | Gemma Terms | not stated for this size |
| `Qwen3-1.7B-q4f16_1-MLC` | 2036.66 | yes | 4096 | Apache-2.0 | yes |
| `Qwen3.5-2B-q4f16_1-MLC` | 2245.44 | no | 4096 | unverified | unverified |
| `Llama-3.2-3B-Instruct-q4f16_1-MLC` | 2263.69 | yes | 4096 | Llama 3.2 Community | no — 8 official languages |
| `Ministral-3-3B-Instruct-2512-BF16-q4f16_1-MLC` | 2863.69 | yes | 4096 | unverified | unverified |

`q4f32_1` twins exist for every row above at roughly 1.3× the VRAM. They are the shape a
device without `shader-f16` would need, and [Not measured yet](#not-measured-yet) says why
that is not the same as a fallback.

The license and language columns are the two that this repository cannot settle by reading
code. Both must be confirmed against the upstream model card before we mirror a single
byte, and the `unverified` rows are unverified rather than unlikely — `Qwen3.5` and
`Ministral-3` are present in the pinned registry and nowhere in our own record.

## What the ordering eliminates

**English-only families, at any size.** SmolLM2, TinyLlama, `phi-1_5`, `phi-2`,
RedPajama, and Gemma's 1B: cheapest rows in the registry, and none of them is a model
Avaia can answer a Ukrainian owner with. Size never buys back a language a model was not
trained on.

**Llama 3.2, at both sizes.** Ukrainian is not among its eight official languages, so it
loses on row 2 before its license is even reached. Its license would have cost us the
`Built with Llama` attribution, a copy of the community license, and the acceptable-use
policy travelling with every mirror we serve.

**Gemma's terms, for the sizes that could have qualified.** Redistribution carries Google's
use policy to whoever receives the weights. That is a term we would be passing on from our
own domain, and 2B does not buy enough to be worth taking it on.

**Reasoning-first checkpoints.** `Ministral-3-3B-Reasoning` and any Qwen3 variant run with
thinking enabled: the adapter disables thinking and caps generation at 128 tokens, so a
model that reasons before answering answers less within the same budget.

## The recommendation

**Keep `Qwen3-0.6B-q4f16_1-MLC` as the model, and serve it from our host.**

It is Apache-2.0, which is the only license in the candidate set that costs our mirror
nothing. Its card claims Ukrainian. It is the smallest multilingual instruct model in the
pinned registry, at 1403 MB of VRAM and `low_resource_required: true`, which is what a
mobile WebView can plausibly grant. Its thinking mode is a switch the adapter already
turns off rather than a checkpoint we would be fighting.

That it is also the foundation's default is convenient, not the argument: the argument is
that redistribution licensing and Ukrainian eliminate everything cheaper, and the surface
ceiling eliminates everything better.

**Second entry, desktop only, after measurement: `Qwen3-1.7B-q4f16_1-MLC`.** Same license,
same family, same switch, 2037 MB. It is the upgrade to offer where a device answers the
probe generously — offered as a distinct declared model, never as a silent substitution.

## What this recommendation does not claim

A 0.6B model's Ukrainian is thin. It will hold a short exchange and it will not hold a long
one. This is acceptable for the first slice for one structural reason: what it produces is
a *proposal*, bounded to 128 tokens, wrapped by `ai` and admitted or refused at the
authority boundary. It is never an authorized message and never a completion.

If the product later needs dialogue quality this size cannot reach, the answer is a
declared remote provider with its own reported transition — the foundation is explicit
that a provider change is reported, not substituted. It is not a bigger local model on a
phone, and it is not a quiet fallback.

## Not measured yet

The foundation's [WebGPU probe](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/probes/webgpu/RESULTS.md)
has four surface rows and every one of them reads `not measured`. Mobile Safari, the
messaging mini-app WebView, and the sandboxed activity iframe are the surfaces this
product is opened on, and none of them has answered.

So the recommendation above is a shortlist conclusion, not a device fact, and three
results would move it:

- **No `shader-f16` on a surface.** No `q4f16_1` model runs there. Per the probe's own
  reading, that surface gets a deterministic product path — not a `q4f32_1` fallback it
  degrades into, which would cost 1925 MB on the weakest device we have.
- **A `maxStorageBufferBindingSize` below what the weights need.** A loading constraint:
  the artifacts get chunked to that ceiling. It does not change which model we chose.
- **No adapter at all.** That surface never renders a local model as available, and Avaia
  is remote or absent there.

Running the probe on the three embedded surfaces is the gate on committing to a mirror.
Mirroring is bytes we pay for; measuring is a page someone opens.

## Serving the artifacts from our host

Independent of which model wins, self-hosting is worth doing on its own terms.

**What the pinned registry points at today.** Weights resolve to `huggingface.co`, and
`model_lib` resolves to
`raw.githubusercontent.com/mlc-ai/binary-mlc-llm-libs/main/web-llm-models/v0_2_84/base/…`.
That path names a *branch*. The WASM our runtime executes is therefore not pinned by our
pin, it is whatever `main` holds at load time, fetched from a host that is not a CDN and
that sees the IP of every person who opens the product. Mirroring makes the pin real.

**What has to be mirrored per model.** `mlc-chat-config.json`, `tokenizer.json`,
`ndarray-cache.json`, every `params_shard_*.bin`, and the `model_lib` WASM.

**The URL shape is not free.** WebLLM normalizes `ModelRecord.model` by appending
`resolve/main/` unless the URL already matches `…/resolve/…/`. A mirror served from a plain
path silently acquires that segment. Serving from `…/<model>/resolve/<revision>/` satisfies
the normalizer and gives us an immutable revision segment to cache forever and to roll
forward explicitly.

**Integrity covers three of the five artifacts.** `ModelRecord.integrity` verifies the
config, the `model_lib`, and the tokenizer files against SRI hashes. The weight shards are
not covered by it. Pinning shard bytes is therefore ours to do — an immutable revision
segment plus the hashes recorded where we record the mirror.

**We can declare the feature gate the registry omits.** The `Qwen3`, `Qwen3.5`,
`Llama-3.2`, `gemma3-1b`, and `Ministral-3` entries carry no `required_features`, while the
older `q4f16_1` entries carry `["shader-f16"]`. A device without f16 therefore downloads
the whole model before failing. Our own `model_list` declares
`required_features: ["shader-f16"]` on every `q4f16_1` entry, which turns that into a
refusal before the first byte.

**Two more things the mirror buys.** Cross-origin isolation and cache headers become ours
to set, and a first load stops being a third-party fetch from our owners' devices.

## Where the choice lives in code

`LocalInferenceRuntime` takes a `modelId` and no `AppConfig`, and `WebLlmBrowserHost`
constructs its engine against WebLLM's prebuilt registry. Neither can be pointed at our
host as it stands, so serving from our own origin needs one of two things:

1. **`nilx-one/web` implements `LocalInferenceHost` itself.** The interface is exported and
   the runtime takes it by injection, so a product host can pass its own `appConfig` to
   `CreateWebWorkerMLCEngine`. Interim, and no foundation change.
2. **`WebLlmBrowserHost` accepts an optional `AppConfig`.** Upstream, product-agnostic, and
   the right home for it: every product that mirrors artifacts needs the same seam.

Either way the model identity — which `model_id`, which revision, which quantization — is
`ai`'s to state and `web`'s to consume, alongside the system prompt and bounded history
`ai` already owns.

## Related

- [Browser-local inference](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/docs/browser-local-inference.md)
- [WebGPU capability probe](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/probes/webgpu/README.md)
- [Integrating a product AI runtime](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/docs/nilx-one-ai-integration.md)

---

© 2026 aiaiaiai · aiaiaiai.org
