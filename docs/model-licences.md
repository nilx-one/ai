# Licences of the local model catalog

Serving weights from our host is redistribution. Nothing is mirrored before its licence is
read, and this document is where the reading is recorded: per `model_id`, what the upstream
card states, what redistribution obliges, and the exact notice text `nilx-one/web` writes into
each mirror's `manifest.json`. The notice strings below are copied verbatim into
`packages/narration-webllm/src/model-catalog.json` there; a change is made here first.

Read on 2026-09-24 from the Hugging Face model cards and API records named below. This is a
reading, recorded so it can be checked, and not legal advice.

## What the conversions do not say

The `mlc-ai/*-q4f16_1-MLC` repositories carry **no licence metadata and no licence file of
their own**. Their cards name a base model and nothing else, and the `OLMo-2` conversion has
no card at all. The licence of each entry below is therefore the base model's, read from the
base model's own card, and the conversion is recorded as what it is: `mlc-ai`'s `q4f16_1`
conversion of that model, published at the commit the mirror pins.

The model library is compiled by `mlc-ai/binary-mlc-llm-libs` under
`web-llm-models/v0_2_84/base/`, Apache-2.0. The pinned registry names one library per entry;
the mirror records its URL and SRI hash.

| `model_id` | base model | card `license:` | base commit read | conversion commit pinned |
|---|---|---|---|---|
| `Qwen3-0.6B-q4f16_1-MLC` | `Qwen/Qwen3-0.6B` | `apache-2.0` | `c1899de2` | `8c14ce481d4c692769976ad52afea453a102df19` |
| `Qwen3-1.7B-q4f16_1-MLC` | `Qwen/Qwen3-1.7B` | `apache-2.0` | `70d244cc` | `80b3abcec6c3b3f5355dc0cc99cc4fb578f192bc` |
| `SmolLM2-360M-Instruct-q4f16_1-MLC` | `HuggingFaceTB/SmolLM2-360M-Instruct` | `apache-2.0` | `a10cc151` | `3a622fd89e0216e8bb10c410c007c786baa8a033` |
| `OLMo-2-0425-1B-Instruct-q4f16_1-MLC` | `allenai/OLMo-2-0425-1B-Instruct` | `apache-2.0` | `48d788ec` | `a3fef630301a7e9a4dcbbdd3a76b0d36b20dc85e` |
| `Llama-3.2-1B-Instruct-q4f16_1-MLC` | `meta-llama/Llama-3.2-1B-Instruct` | `llama3.2` | `92131767` (gated) | `2a37b0a5ecb622d51ddc2fac74de0b95872affd7` |

## Apache-2.0: four entries

Qwen3-0.6B, Qwen3-1.7B, SmolLM2-360M-Instruct and OLMo-2-0425-1B-Instruct. Qwen3's card
links a `LICENSE` file; SmolLM2 and OLMo 2 state Apache-2.0 in the card and ship no licence
file. None of the four upstream repositories carries a `NOTICE` file.

What redistributing them obliges (Apache-2.0 §4):

- **(a) a copy of the licence** with the weights. The mirror places `LICENSE`, the Apache-2.0
  text, in every revision directory it serves. This was missing from the first mirror, which
  carried a sentence pointing at the upstream card instead.
- **(b) modified files say so.** We serve the conversion unmodified. The conversion is
  `mlc-ai`'s modification, and the notices name it.
- **(c) attribution notices from the source are retained**, which the notices below do.
- **(d) a `NOTICE` file travels with the work if the work has one.** None does.

Nothing has to be displayed in the product, and nothing reaches the name of a model trained on
their outputs.

Notice text, per entry, as `manifest.json` carries it:

- `Qwen3-0.6B-q4f16_1-MLC`
  - `Qwen3-0.6B by Qwen, licensed under the Apache License 2.0; a copy of the licence is served beside these weights as LICENSE.`
  - `Converted to MLC format (q4f16_1) and published by mlc-ai: https://huggingface.co/mlc-ai/Qwen3-0.6B-q4f16_1-MLC`
  - `Model library compiled by mlc-ai/binary-mlc-llm-libs (Apache-2.0).`
- `Qwen3-1.7B-q4f16_1-MLC`
  - `Qwen3-1.7B by Qwen, licensed under the Apache License 2.0; a copy of the licence is served beside these weights as LICENSE.`
  - `Converted to MLC format (q4f16_1) and published by mlc-ai: https://huggingface.co/mlc-ai/Qwen3-1.7B-q4f16_1-MLC`
  - `Model library compiled by mlc-ai/binary-mlc-llm-libs (Apache-2.0).`
- `SmolLM2-360M-Instruct-q4f16_1-MLC`
  - `SmolLM2-360M-Instruct by Hugging Face (HuggingFaceTB), licensed under the Apache License 2.0; a copy of the licence is served beside these weights as LICENSE.`
  - `Converted to MLC format (q4f16_1) and published by mlc-ai: https://huggingface.co/mlc-ai/SmolLM2-360M-Instruct-q4f16_1-MLC`
  - `Model library compiled by mlc-ai/binary-mlc-llm-libs (Apache-2.0).`
- `OLMo-2-0425-1B-Instruct-q4f16_1-MLC`
  - `OLMo-2-0425-1B-Instruct by the Allen Institute for AI (Ai2), licensed under the Apache License 2.0; a copy of the licence is served beside these weights as LICENSE.`
  - `Ai2 states that OLMo 2 is intended for research and educational use: https://allenai.org/responsible-use`
  - `Converted to MLC format (q4f16_1) and published by mlc-ai: https://huggingface.co/mlc-ai/OLMo-2-0425-1B-Instruct-q4f16_1-MLC`
  - `Model library compiled by mlc-ai/binary-mlc-llm-libs (Apache-2.0).`

OLMo 2's "intended for research and educational use" is a statement beside the licence, not a
term of it: Apache-2.0 carries no field-of-use restriction. It is carried as a notice so that
a person choosing the entry reads it too.

## Llama-3.2-1B-Instruct: Llama 3.2 Community License

The base repository is gated behind manual approval and holds `LICENSE.txt` and
`USE_POLICY.md`. The conversion is not gated and holds neither, so a mirror built from the
conversion has to supply what the conversion leaves out. The licence was read from
`meta-llama/llama-models`, `models/llama3_2/LICENSE` (Version Release Date September 25, 2024;
sha256 `8cc15535a8a34b41888f644b339a1a9eb428af793a4f5e24df58a3e5b1487d74`), and the policy from
`models/llama3_2/USE_POLICY.md` beside it.

What redistributing it obliges, and where each obligation is kept:

1. **A copy of the Agreement with the weights** (§1.b.i(A)). The mirror places `LICENSE` in the
   revision directory. `nilx-one/web` vendors the text at
   `deploy/web/third_party/llama-3.2/LICENSE` and the bootstrap refuses to write it unless its
   sha256 matches the one above.
2. **"Built with Llama", displayed prominently** on a related website, user interface,
   blogpost, about page or product documentation (§1.b.i(B)). The product has no about
   surface, so it is shown in Settings beside the entry and stated in `nilx-one/web`'s
   `docs/local-models.md`.
3. **The Notice text file** (§1.b.iii), with exactly this sentence:
   `Llama 3.2 is licensed under the Llama 3.2 Community License, Copyright © Meta Platforms, Inc. All Rights Reserved.`
   The mirror places it as `NOTICE` in the revision directory, and it is the first of the
   manifest's notices.
4. **The Acceptable Use Policy, incorporated by reference** (§1.b.iv):
   `https://www.llama.com/llama3_2/use-policy`. It is linked where the entry is offered. A
   breach of the policy is a breach of the licence.
5. **The naming clause** (§1.b.i, last sentence): a model created, trained, fine-tuned or
   otherwise improved with Llama Materials *or their outputs*, and then distributed, must have
   a name that begins with "Llama". Narration this entry wrote is Llama output, so it is marked
   where it is produced and excluded from any training egress — see
   [the egress policy](presence-journal-egress-policy.md). `Licence::names_derived_models` in
   `src/inference.rs` states the same thing in code.
6. **The 700 million monthly-active-user threshold** (§2) does not apply at this product's
   scale; it is recorded because it is a term.
7. **The mark** (§5.a): "Llama" may be used as §1.b.i requires and to describe what we
   redistribute, under Meta's brand guidelines, and not otherwise.
8. **Termination** (§6): bringing an intellectual-property claim against Meta over Llama
   Materials ends the licence, after which the weights are deleted from the mirror. Governing
   law and venue are California (§7).

Notice text, as `manifest.json` carries it:

- `Llama 3.2 is licensed under the Llama 3.2 Community License, Copyright © Meta Platforms, Inc. All Rights Reserved.`
- `Built with Llama.`
- `Use of Llama 3.2 is subject to its Acceptable Use Policy: https://www.llama.com/llama3_2/use-policy`
- `A copy of the Llama 3.2 Community License is served beside these weights as LICENSE.`
- `Converted to MLC format (q4f16_1) and published by mlc-ai: https://huggingface.co/mlc-ai/Llama-3.2-1B-Instruct-q4f16_1-MLC`
- `Model library compiled by mlc-ai/binary-mlc-llm-libs (Apache-2.0).`

## Not read yet

- Card changes after the commits above are not tracked. Pinning the conversion commit keeps
  the bytes fixed; it does not keep the card's claims fixed.
- The model library is still fetched from `binary-mlc-llm-libs` on `main`, as the pinned
  registry names it. Its SRI hash is recorded per mirror revision; its commit is not.

---

© 2026 aiaiaiai · aiaiaiai.org
