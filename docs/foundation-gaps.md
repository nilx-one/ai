# What belongs upstream, and what does not

`nilx-one/ai` consumes [`aiaiaiai-org/artificial-intelligence`](https://github.com/aiaiaiai-org/artificial-intelligence)
as a library. The dividing rule is the foundation's own: it owns product-agnostic AI
execution primitives and refusals; this repository owns 0x1 vocabulary and product policy.
Building the first on-device slice found four places where work sits on the wrong side of
that line — three that should move up, and a list of things that must not.

Licensing runs the right way for all of it: an Apache-2.0 foundation consumed by an MPL-2.0
product, so code moving upstream is relicensed Apache-2.0 by us as its authors, and code
moving the other way could not be.

## 1. The adapter accepts no `AppConfig`

`WebLlmBrowserHost` builds its engine with `CreateWebWorkerMLCEngine(worker, modelId, {
initProgressCallback })` and asks `hasModelInCache(modelId)` with no second argument. Both
therefore resolve against `WebLLM`'s prebuilt registry and its default cache backend.

Every product that mirrors model artifacts needs all three of the things that argument
carries, and none of them is product-specific:

- `model_list` entries pointing at the mirror rather than at `huggingface.co` and at a
  mutable `raw.githubusercontent.com` branch;
- `required_features`, which the pinned registry omits on its newer entries, so a device
  without `shader-f16` downloads a whole model before failing to initialize it;
- `integrity`, the SRI hashes for config, `model_lib`, and tokenizer;
- `cacheBackend`, one of `cache`, `indexeddb`, `cross-origin`, `opfs` — and a cache lookup
  made without it answers about a different place than the load will use.

Proposed shape: an optional `AppConfig` on the host, passed to both the engine construction
and the cache query. Nothing about it names a product.

## 2. The adapter accepts no `response_format`

`GenerationOptions` carries `maxTokens`, `temperature`, and `topP`. The pinned `WebLLM`
supports `response_format` with `json_object`, `grammar` (EBNF), and `structural_tag`, and
`stream()` passes none of them.

A product whose model *chooses* rather than *talks* needs that seam, and this is not an Avaia
peculiarity: the foundation's whole shape says a model proposes and an authority disposes, so
constraining a proposal to the set an authority could even consider is the foundation's kind
of refusal. With it, an unresolvable choice is unrepresentable rather than merely rejected.

What stays here is the content: the action vocabulary, the grammar text, and the parse are
0x1 semantics and live in `src/decision.rs`. What moves up is the ability to hand a decode
its constraint.

## 3. The probe measures four things and two hard floors are not among them

The pinned runtime asks a device for `maxBufferSize` and `maxStorageBufferBindingSize` of
1 GiB, falls back once to 256 MiB and 128 MiB, and throws below those. It also requires
`maxComputeWorkgroupStorageSize` of 32 KiB and `maxStorageBuffersPerShaderStage` of **10**,
each with no fallback at all — and 10 is above the `WebGPU` default of 8, so a current,
healthy device can clear every recorded row and still fail to initialize.

`probes/webgpu` records WebGPU availability, `shader-f16`, `maxBufferSize`, and
`maxStorageBufferBindingSize`. Adding the two missing limits is a few lines in the probe page
and two columns in its results table, and it is the cheapest of these three changes by a wide
margin. It is also the one that changes what a measured row *means*: without them, a row can
read "supported" for a surface the runtime refuses to start on.

The floors themselves are a candidate to move with it. This repository currently quotes them
in `inference::runtime_floor` because selection has to refuse on them, but they are facts
about the foundation's own adapter and its pinned `WebLLM`, not about 0x1 — and a copy of
them in every product is a copy that drifts when the pin moves. The same argument covers a
typed capability value: the shape of what a probe reports belongs beside the probe.

## What must not move up

- **The catalog.** Which models this product serves is product policy, and the foundation
  deliberately pins one default only so its adapter has a tested pair.
- **`MemoryBudget`.** A declared willingness to spend memory on a surface is a product's
  claim about its own product, and it must not be dressed up as a measurement anywhere.
- **`MapTargetId`, `AvaiaActionProposal`, and the action names in the grammar.** 0x1
  vocabulary. The foundation contains no participant, relationship, interaction, or record
  type on purpose, and a navigation verb is exactly that kind of type.
- **Anything that models completion.** The foundation stops at an effect request, and 0x1
  and `core` own whether anything happened. Nothing here changes that.

## Until then

`LocalInferenceHost` is an exported interface and `LocalInferenceRuntime` takes it by
injection, so `nilx-one/web` can implement a host that passes its own `appConfig` and its own
`response_format` today. That is the interim path for gaps 1 and 2, and it is the same code
that gets deleted when the seams land upstream.

## Related

- [Choosing the models Avaia loads](model-selection.md)
- [Browser-local inference](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/docs/browser-local-inference.md)
- [The WebGPU capability probe](https://github.com/aiaiaiai-org/artificial-intelligence/blob/master/probes/webgpu/README.md)

---

© 2026 aiaiaiai · aiaiaiai.org
