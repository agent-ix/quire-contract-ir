---
id: SR-2026110602354601
title: "IR-1352 PR 921 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1352; PR #921; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Reviewed exact PR head e48a5469a5cca950fb735f13f4c322bdaab8c9df against FR-038-AC-473, QSpec FR-440, and existing protocol-slot scope rules. The five new tests call the public reader and assert retained values or typed refusals. The QSL producer probe and final Cargo gate belong to the lead.

## Verdict

**CONDITIONAL** — case-arm binder identity and scope paths appear sound, but the new pre-scan does uncharged traversal and allocation before the work meter can stop it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The case-arm pre-scan traverses and allocates all arm and binder entries before charging them; a large valid case can exceed a caller's remaining work allowance before the first stop. Charge while parsing each entry and stop before allocating the next one. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:4128 |

## Examined scope

- role: examined; id: FR-038-AC-473; excerpt: "An IR-owned admitted-wire control with two arms binding distinct nodes under the same name admits reads of each binder only inside its own arm body. A body reference to the other arm's binder, including one in a shared body node reached under both arm scopes, and a reference to an arm binder from the scrutinee or an enclosing sibling expression each refuses `missing_declaration`/`missing-name` at the protocol owner with `BinderScope` detail naming that binder, slot owner, stage and `read`."
- role: examined; id: FR-038; excerpt: "Untrusted wire bytes; caller-selected read limits, for which the crate ships one named finite default appropriate to a single local request — 1048576 bytes, 10000 nodes, 100000 edges, 100000 occurrences, 10000 diagnostics and 1000000 term-validation visits — as stable API, every member finite so that the default admits no unbounded read"
- role: context_only; id: FR-038-AC-465/466; excerpt: "A protocol slot's lexical let/query/count/sum binders are in scope only inside their bound expression bodies, including nested binders and shared graph nodes reached under different lexical scopes."

Plan completion: not assessed. Cargo was intentionally not run because the shared target lock was occupied.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 at PR head `38008979264d250881f0ed98ffa81b8e93364e9b`: each arm is charged before shape inspection and each binder before lookup and insertion into `binder_positions`; uncharged pre-collection was removed. |

## Disposition round 1

**PASS for this finding.** The fix preserves one charge per arm and binder, now before the associated parsing and allocation. No new finding in the fix diff. Final scoped Cargo gate remains with the lead.
