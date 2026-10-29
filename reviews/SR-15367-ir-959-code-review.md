---
id: SR-15367
title: "Code review of IR-959 PR 506 manifest requirements"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; Cargo.toml:72-75, crates/quire-contract-model/Cargo.toml:39-45,69"
review_set: subset
---

## Summary

Ticket: IR-959. Reviewed the six changed first-party Git version requirements and the two explanatory comments at PR 506, against the reviewed dependency state. The manifest diff is confined to those entries. The Cargo.lock blob is identical at both commits; locked metadata resolves. In an isolated worktree, `cargo update -p ix-trace-rs` fails at the base requirements because the current branch offers 0.1.2, and succeeds at the reviewed head. Updates of quire-canonical and quire-verification-contracts also succeed at the reviewed head.

## Scope

- Cargo.toml:72, examined: `# Moving first-party branches use compatible ranges; Cargo.lock records selected revisions.`
- Cargo.toml:73, examined: `ix-trace-rs = { version = "0.1", git = "https://github.com/agent-ix/ix-trace-rs", branch = "main" }`
- Cargo.toml:74, examined: `quire-canonical = { version = "0.3", git = "https://github.com/agent-ix/quire-canonical", branch = "main" }`
- Cargo.toml:75, examined: `quire-verification-contracts = { version = "0.1", git = "https://github.com/agent-ix/quire-verification-contracts", branch = "main" }`
- crates/quire-contract-model/Cargo.toml:39, examined: `# Moving first-party branches use compatible ranges; Cargo.lock records selected revisions.`
- crates/quire-contract-model/Cargo.toml:40, examined: `quire-verification-contracts = { version = "0.1", git = "https://github.com/agent-ix/quire-verification-contracts", branch = "main" }`
- crates/quire-contract-model/Cargo.toml:45, examined: `quire-canonical = { version = "0.3", git = "https://github.com/agent-ix/quire-canonical", branch = "main", features = ["serde_json"] }`
- crates/quire-contract-model/Cargo.toml:69, examined: `ix-trace-rs = { version = "0.1", git = "https://github.com/agent-ix/ix-trace-rs", branch = "main" }`

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

**PASS** — The six requirements use the intended compatible ranges, retain each Git source, branch, dependency role, and feature setting, and resolve against the current branches. The two comments accurately describe the lockfile's role. `git diff --check` and `cargo metadata --locked --format-version 1 --no-deps` pass. The installed quire 0.36.2 has no `check-lock` subcommand, so the IR-736 shared lock-coherence check was unavailable here. Final CI is outside this scoped review.
