# Contributing to 0x1 AI

Contributions are welcome. AI behavior may propose, plan, simulate, or select allowed actions, but it must remain behind the 0x1 protocol boundary.

## Before Changing Code

Read the canonical contracts in `nilx-one/0x1` and deterministic shared behavior in `nilx-one/core` first. Model output is not authority, a proposal is not interaction completion, and simulation is not a BondChain fact.

Keep contributions narrowly scoped. Explain the problem, preserve authority boundaries, and include verification appropriate to the affected surface.

## Pull Requests

Prefer one coherent task per pull request. State what changes, why it is needed, which contract owns the behavior, what was verified, and whether compatibility, licensing, security, model/provider behavior, or migration changes.

## Source Licensing

New authored source and configuration files that support comments must begin with the canonical repository header for their file format. For Rust and other `//`-comment formats:

```text
// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: MPL-2.0
```

Use equivalent comment syntax for other formats. Do not inject headers into JSON, lockfiles, generated output, vendored third-party files, snapshots, or formats where comments would break the contract.

Run `python scripts/check_repository_policy.py MPL-2.0` before submitting. Required GitHub CI runs the same check.

## Contribution Rights

Contributors keep ownership of their original contributions.

By intentionally submitting work for inclusion in this repository, the contributor is expected to provide the rights described in [CLA.md](CLA.md). The intended grant lets the project integrate, modify, distribute, sublicense, and relicense accepted work while leaving the contributor free to use their original contribution elsewhere.

The CLA is currently provisional until a production acceptance mechanism is finalized. Maintainers may require explicit signed or electronic acceptance before merging an external contribution.

## Third-Party Material

Do not introduce code, model artifacts, datasets, generated material, assets, or dependencies whose terms conflict with MPL-2.0 or with the project's ability to distribute accepted work. Identify externally sourced model/data terms explicitly and preserve required notices.

## Product Identity

Open-source permission does not grant permission to present a derivative as official 0x1. See [TRADEMARKS.md](TRADEMARKS.md).

---

© 2026 aiaiaiai · aiaiaiai.org
