---
id: SR-1555
title: "EARS conformance review of quire-contract-ir PR #295 (IR-627 anonymous structural node bodies)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@984c283099ce117b5ab7cba2b8f03fe3d6e5e5bc; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (section 'Anonymous structural node bodies and keys')"
review_set: subset
---

## Summary

Ticket: IR-627. Two new normative statements were checked: the decided body check and the gated
re-derivation. Both use event-driven `When ... the reader shall` grammar with a named response. Each
packs several `shall` clauses (read, compare, node shape, equality; re-derive, refuse, extend, one
function) into one paragraph, and the refusal sentence is indicative, with no `shall`.

## Verdict

Approved with one low finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Both normative paragraphs are compound: the decided one has three `shall`s plus an indicative refusal ("A node that fails any of these refuses ..."), and the gated one has five, including a design constraint ("shall be the same function"). Split them into atomic statements and state the refusal as a `shall` | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1776-1809 |

## Dispositions

Round 1, reviewed at ab860cac3a7162c5eea073d99bb5bcf5433e1237.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ab860cac3a7162c5eea073d99bb5bcf5433e1237 |
