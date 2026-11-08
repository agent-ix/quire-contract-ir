---
id: SR-202611070951309518
title: "IR-1404 PR 978 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 978; intake.rs, v2/mod.rs, checked_package_v2_artifact_refs.rs; FR-038-AC-53/510/511 subset"
review_set: subset
---

## Summary

Ticket: IR-1404. Independent frozen-candidate review of PR 978. Code review includes the Rust lane and repository AGENTS.md/CLAUDE.md. No applicable AssuranceProfile exists under spec/. The source branch remains untouched.

The production change moves exactly the owner/owner_model UnknownMember classification from an owned-pointer mapper into the existing borrowed intake policy. The old mapper is removed, with no second rule, public API, local budget, allowlist, dependency change or CI weakening. The dependency-entry mapper remains called unchanged.

## Verdict

PASS for the frozen PR subset; no introduced defects found. Existing repository-wide matrix gaps and the remaining deferred whole-read cases are not qualified or closed by this review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-53 (examined): A `lock.sources` row carrying `digest_domain` or `digest` refuses as `unknown_member` at the first extra member in document order, even when an `authority` or `identity` value is empty; no source digest is checked or retained.
- FR-038-AC-510 (examined): When the diagnostic's own request is denied, or the denied request was a refusal diagnostic, the read returns `Exhausted` holding the original denied resource and amount, its configured bound (including zero), its `PackageRead` locus and the pending outcome class (refusal code or structural limit kind), and makes no further request.
- FR-038-AC-511 (examined): For every read, the read-storage delta the budget admitted equals the total bytes requested during the call; no request is observed for a denied charge; and every request observed after a denial belongs to the charged outcome diagnostic.

Reviewed the full three-file diff, the common decoder caller, first-fault snapshot, refusal reservation, and dependency-entry mapper. For an owner prefix at any tracked length 4 through 12, including a truncated deeper trail, the slice pattern requires the same first four segments and shrinks to length 4 before reservation. No suffix writes execute for that case. Initially MalformedWire faults retain ordinary suffix enrichment; unrelated namespaces retain UnknownMember; dependency selections cannot match the owner prefix. Code/cause/primary path and document-order precedence remain as before on affordable reads. An unaffordable owner diagnostic now correctly records pending MalformedWire before returning without allocation.

Accepted existing bounded native evidence after inspecting assertions and logs: schema-policy suite 6 passed; unchanged original lock-source-digest control 1 passed; workspace all-target Clippy with conformance-cli,test-support and -D warnings passed. Diff whitespace independently passed. No new Cargo job, whole binary, ignored suite, Kani or aggregate gate ran. Main fetched during review is 405e88b3; changes after brief main 2fc35a9a touch only temporal.rs and its tests, with no changed classifier or reader caller. PR still draft and frozen at fa59aa23.

Computed the entire repository matrix with quire 0.38.1: 1023 tagged, 96 untagged, 26 method-without-symbol and 8 tagged-by-ignored-test criteria. These pre-existing repository gaps remain open; this scoped PR verdict is not a repository-wide assurance PASS or closure of FR-038-AC-511. AC-53, AC-510 and AC-511 compute tagged; AC-53 now binds the restored original test. New inline qualified trace attributes are not surfaced as binders by this installed matrix, so their source tags and actual native execution were checked directly. No new criterion, untraced public behavior, stub, copied rule or hollow test is introduced. Reverse inventory for the changed behavior: one existing public budgeted reader, one internal classifier and its retained dependency classifier; zero new APIs, unspecified behaviors or stubs. Semantic intent comparison is included in the authorized independent code review; optional repository-wide semantic analysis was not run.

Plan completion: not assessed
