---
id: SR-670
title: "spec review of PR 241 (AD-005 IR to QSL seam, AD-006 IR to codegen seam)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@43967a9b8410c5825ffef77df48ff45bc265f23b; spec/assurance/AD-005-qsl-consumption-seam.md, spec/assurance/AD-006-codegen-consumption-seam.md, spec/spec.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-005
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/AD-006
    type: reviews
---
# SR-670: spec review of PR 241

## Summary

Ticket: IR-323. Reviewed head 43967a9 (the planner-answers fold on top of d784ff7). Examined
both ADs in full and the spec.md registry and References edits. Checked them for
ArchitectureDescription structure, traceability of every cited FR, AC and TC id, consistency
with AD-001, FR-019, FR-028, FR-030 and FR-039 at origin/main 0a889f9, and with the sibling ADs:
IR AD-004 (PR 239, a7924dc), codegen AD-002 and AD-003 (codegen PR 214), codegen AD-004 (codegen
PR 215) and runtime AD-003 (runtime PR 92). Also checked confidentiality, and checked for pins,
SHAs, version records, compatibility layers and copies between repos. `git merge-tree` against
PR 239 is clean. `quire validate` passes. `make spec` reports the same 17 unbacked rows as main,
and none of them comes from this diff. No requirement id is minted: D-1..D-6, G-1..G-7 and
R3-Q1..Q5, R3-C1..C4 are stated as local labels and routing ids. No compatibility layer or
vendoring is proposed. The measured claims about the code are reviewed in SR-671 (gap analysis).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AD-006 Decisions B and D give opposite orders. B says codegen takes the lowerings first and IR deletes them afterwards. D says IR changes first and codegen follows "in the same step", and A says the glob removal and codegen's direct dependency land "in the same change". A change across two repos cannot land in one change, and D contradicts B. Two implementers would sequence IR-347 differently | spec/assurance/AD-006-codegen-consumption-seam.md:117-128 |
| FND-002 | medium | AD-006 conflicts with sibling codegen ADs and does not mention them. Decision B and R3-C1 move every Kani family lowering into codegen. Codegen AD-004 (PR 215) maps `bounded_collections`, `definedness_arithmetic` and `finite_reference_graphs` into `kani/generate/corpus/` and says "they forward to IR lowerings". Codegen AD-003 (PR 214) says "The seam to IR is IR's root crate API", which is the opposite of Decision A | spec/assurance/AD-006-codegen-consumption-seam.md:120-123,192 |
| FND-003 | medium | AD-006 says whether the `BoundPackage` projection path is retired "is not stated in either specification", and routes it as an open question and R3-C3. Codegen AD-004 (PR 215) records the IR-311 ruling: `CheckedPackageV2` is the one input model and V1 `BoundPackage` is retired entirely. The question is already answered in a sibling AD and should be cited, not routed | spec/assurance/AD-006-codegen-consumption-seam.md:170-171,180,194 |
| FND-004 | medium | The IR-274 row says "R1, due 09-30 per the IR-274 text, untrusted", and Decision C names "QSL Architecture Remediation R1". Measured: IR-274's text has no date and its dueDate is null. 2026-09-30 is the target date of the Linear milestone "R1 — quire-canonical, ecosystem-wide". The source is misattributed, the date has already passed, and a milestone label and date record delivery order in a public spec | spec/assurance/AD-005-qsl-consumption-seam.md:117,182 |
| FND-005 | medium | The ADs record commit ids of the measuring revisions in prose: QSL d81193f9, IR 0a889f9 and codegen 2fad745. The repo CLAUDE.md forbids introducing SHAs, and IR-323 says no SHAs. AD-004 avoids this by writing "measured at the commit this AD was written against" | spec/assurance/AD-005-qsl-consumption-seam.md:50; spec/assurance/AD-006-codegen-consumption-seam.md:51,80 |
| FND-006 | medium | The ADs rest on documents that are not on main. They make 11 references to AD-004 (PR 239), and AD-005:31-32 hands the whole reader seam to it. They also cite codegen AD-002 and AD-003 (codegen PR 214) and runtime AD-003 (runtime PR 92). If this PR merges first, those references dangle. "R-I3" (AD-006:119) is not defined anywhere in AD-004's text | spec/assurance/AD-005-qsl-consumption-seam.md:31-32,40; spec/assurance/AD-006-codegen-consumption-seam.md:31-32,42,119 |
| FND-007 | low | The cargo-deny guard is by name only. The bans recommendation lists "each QSL crate by name", which goes stale when QSL adds a crate, and D-2 drops the "by source" half that D-1 keeps. tc_041's `qsl-` prefix and git-source check is stronger than a name list. The AD should keep the source or prefix check beside the bans | spec/assurance/AD-005-qsl-consumption-seam.md:133-135,183 |
| FND-008 | low | AD-006 divergence 2 attributes the `KaniProvider*` problem only to AD-001 "which gives the map to codegen". FR-039 "Items QSL owns" assigns `KaniProviderResult` and `KaniProviderRecord` to QSL's `TerminalValue` and `TerminalRecord`. AD-001:56 still lists "the Kani outcome to QSL terminal-value map" in IR's `kani` module, which contradicts AD-001:98 and :108. The seam AD should name both | spec/assurance/AD-006-codegen-consumption-seam.md:157-160 |
| FND-009 | low | AD-006 Decision C cites FR-030-AC-5 for "exported constants or a typed cause". FR-030-AC-5 covers the constructors and struct literals only. The open question on the same subject is still listed, and its recommendation picks "typed cause", so the decision and the open question disagree | spec/assurance/AD-006-codegen-consumption-seam.md:124-126,181 |
| FND-010 | low | AD-006 says the move of the lowerings is "inside IR-347's reopened scope". IR-347's text (untrusted ticket text) names KaniProviderResult, witness and replay, but not the lowerings. IR-343's text puts "where the Kani lowerings live (per the ruling)" in the layout AD. The basis is the planner's decision, which the AD does not cite, and the ticket text has not been brought into line. Its parenthesis also says `KaniProvider*` is "used by codegen", against AD-006:160 | spec/assurance/AD-006-codegen-consumption-seam.md:122-123,162-163 |

## Verdict

Changes needed. The structure is sound: boundary, views, decisions, testable invariants, risks,
open questions and routed needs with owners. Routed gaps are needs, not decisions. IR depends on
QSL nowhere. Nothing confidential from quire-research appears. FND-001 to FND-006 should be fixed
before merge. FND-006 at least makes PR 239 a merge predecessor.
