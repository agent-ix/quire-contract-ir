---
id: SR-653
title: "gap analysis of PR 239 against IR-324 (QSpec to IR seam AD)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@a7924dc2c05d421e4323a28373dc0c546ec9917a; spec/assurance/AD-004-checked-package-seam.md, spec/spec.md"
review_set: subset
---
# SR-653: gap analysis of PR 239 against IR-324

## Summary

Ticket: IR-324. The PR is spec only, so this is an acceptance check of the AD against the
ticket's scope and the dispatching brief. Plan completion was not assessed.

| Acceptance item | Result |
| --- | --- |
| One AD for the QSpec to IR seam (`quire.checked-package/v2`, FR-038/040/344), owned by the repo that owns the concept | present: AD-004, in IR |
| QSL- and QSpec-owned gaps routed with a stated need, not decided | present: R-Q6 and R-S1..R-S8 (see SR-652 for the off-seam rows) |
| No requirement ids minted | met: Q-1..Q-9 are local labels |
| No pins, SHAs, version records, compatibility layers or vendoring | met: no SHA, and "No compatibility layer is proposed" |
| Only digest is the single canonical content-identity digest | not met as worded (FND-001) |
| No research findings or roadmap in the public repo; no quire-research internals | met |
| `make spec` adds no unbacked row | met: still 17, all pre-existing |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The AD says "`package_id` is the one canonical content-identity digest on this seam". The next sentence describes a second one: the `sha256-jcs` digest of each domain package document, which IR recomputes. The wire also carries application node keys (the JCS SHA-256 of each preimage, refused as `invalid_package`/`stale-node-key`), nominal identity digests, the dependency `package_id`s and the lock's raw-artifact digests. Say instead that `package_id` is the package's content identity, that the other digests are QSpec-owned content identities of its parts, and that IR adds none | spec/assurance/AD-004-checked-package-seam.md:67-71 |

## Verdict

Changes needed on FND-001 only. All other acceptance items are met. Approve as a DRAFT AD once
FND-001 is fixed and SR-651 and SR-652 are addressed. Merge stays gated on the planner's QSL
and QSpec routing review.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | dbf1475: "`package_id` is the package's content identity". The other wire digests are listed as "QSpec-owned identities of the package's parts, and IR adds none" |
