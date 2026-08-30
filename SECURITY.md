# Security Policy

Security reports are welcome and should be handled privately when disclosure could expose users, infrastructure, credentials, model-provider secrets, or exploitable behavior.

## Reporting

Prefer GitHub private vulnerability reporting or a private security advisory when available. Do not publish exploit details in a public issue before maintainers have had a reasonable opportunity to assess and remediate the problem.

A useful report identifies the affected commit or surface, impact and realistic preconditions, safe reproduction steps, and whether the issue concerns model/provider behavior, runtime authority, tool execution, data exposure, or protocol integration.

Never include live credentials, API keys, private prompts or memories, access tokens, private user data, or secrets in reports or fixtures.

## Authority Boundary

A security fix must not manufacture consent, reciprocity, authorization, or completed interaction facts. AI remains behind the protocol boundary; canonical protocol and deterministic core behavior remain authoritative.

Third-party model, dataset, runtime, and dependency vulnerabilities should identify the affected component and version/range when known.

---

© 2026 aiaiaiai · aiaiaiai.org
