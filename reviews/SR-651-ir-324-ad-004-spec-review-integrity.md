---
id: SR-651
title: "spec review (integrity) of PR 239 (AD-004 QSpec to IR checked-package seam)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@a7924dc2c05d421e4323a28373dc0c546ec9917a; spec/assurance/AD-004-checked-package-seam.md, spec/spec.md"
review_set: subset
---
# SR-651: integrity review of AD-004 (PR 239)

## Summary

Ticket: IR-324. Every file and line citation and every factual claim in AD-004 was checked
against IR `origin/main` (0a889f9, which is the PR's merge base). The PR changes only the AD
and two lines of `spec/spec.md`.

These claims are correct:

- The version dispatch in `dispatch.rs`: `malformed_wire` for a missing or non-string
  `contract_version`, `unknown_contract_version` for any other version.
- The `stale_dependency` refusals at `v2/mod.rs:760-765`, `:866-868` and `:1833-1843`. The
  `invalid_semantic_graph` refusal for nominal identity at `v2/identity.rs:411`.
- The `bounded()` limits and `MAXIMUM_DEPTH = 16_384` (`shared.rs:43-55`).
- The seven `CompleteLoweringRecordV2` kinds.
- The `evidence.rs` doc.
- `src/lib.rs:12` and `src/kani/mod.rs:23`.
- The FR-038 AC numbering hole (no AC-16 and no AC-34).
- The FR-038 depth deviation (STD-125) and the FR-040 `record_value_type` deviation.
- FR-344's split of two members carried and five refused.
- The QSL dependency name and `tc_469_step_6_the_emitted_package_admits_via_i04`.
- CG's glob imports (`routed_generation.rs:20`) and the driver's model-crate import.
- The duplicate QSpec `FR-341`.
- IR-347's reopened scope.

`make spec` passes structural validation. Strict coverage reports the same 17 unbacked rows as
main, and none of them comes from this change. The registry row and References edits are
correct and do not textually conflict with sibling PR 241 (AD-005 and AD-006).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Q-8 says each of "the seven ADR-002 members FR-344 names as unadmitted" refuses with a typed code. FR-344 names five as unadmitted. The other two (operation frame and embedded meaning vocabulary) are admitted, and the AD says so itself at :153-154. A test written from Q-8 would fail on the admitted operation frame | spec/assurance/AD-004-checked-package-seam.md:133-134 |
| FND-002 | medium | Q-1 to Q-6 are not marked as current or gap. Only Q-7 and Q-8 say "existing". Q-1's property test over mutated packages does not exist: IR has no proptest or fuzz dependency. Q-6 (closed vocabularies decoded into an enum once, with no catch-all arm) is unmeasured, and bodies are validated over `serde_json::Value` with `_ =>` arms (for example `v2/frame.rs:118`) | spec/assurance/AD-004-checked-package-seam.md:118-130 |
| FND-003 | low | Q-7 credits FR-037-AC-6 with "no ... terminal type" and "depends on no QSL crate". FR-037-AC-6 covers neither. The no-QSL-dependency rule is FR-028, which binds the model crate only, and no requirement states it for the root crate | spec/assurance/AD-004-checked-package-seam.md:131-132 |
| FND-004 | low | The AD says `contract_version` is read "before any other decode". Byte limit, depth scan, strict JSON parse with duplicate-member detection, and the canonical-bytes check all run first (`common.rs:225-259`). So an unknown version with a duplicate member refuses `duplicate_member`. FR-038 says "before any version-specific decoding", as Q-2 does | spec/assurance/AD-004-checked-package-seam.md:58-59 |
| FND-005 | low | The failure table puts `duplicate_member` and `unknown_member` in one row with the byte-stream refusals and says "no pointer for the byte stream". Both carry a pointer (`common.rs:914`, `:1034`; `decode_closed` at `common.rs:317-325`). The table also has no row for an absent selected document, which refuses `missing_import` | spec/assurance/AD-004-checked-package-seam.md:87-88 |
| FND-006 | low | Precedence rule 2 and the failure table rely on "QSpec's reference reader" but never say what or where it is. It is a test harness in QSpec. Rule 2 also reads as a ruling on QSpec's authority. It should be stated as IR's policy: IR follows that harness where the text is silent | spec/assurance/AD-004-checked-package-seam.md:94, 106-107, 187 |
| FND-007 | low | "IR refuses every text-admission node" is stated as measured, but no IR spec or code defines "text admission" and the claim cites nothing. The term comes from codegen. The owner is also inconsistent: Open questions says "owner decision", while R-S8 routes it to QSpec | spec/assurance/AD-004-checked-package-seam.md:162-165, 190, 218 |
| FND-008 | low | The ids R-I1, R-I2 and R-I3 are never defined in this AD, though sibling PR 241's AD-006 cites "sibling PR 239 routes it as R-I3". Map each id to its bullet at :175-181 | spec/assurance/AD-004-checked-package-seam.md:220-221 |

## Verdict

Changes needed. Every factual claim about the reader's refusal codes, the reader path, the
glob, the `KaniProvider*` exports, the depth cap and the FR-040 and FR-344 deviations matches
`origin/main`. The defects are confined to the invariant list and a few imprecise sentences,
and all are text fixes. No requirement id is minted and the AD adds no unbacked row.

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-contract-ir@dbf1475a5181be0257f293aeabaa70a472579560.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | low | The fix inserted "matching the QSpec's reference reader", a doubled determiner, and left an unwrapped line of about 140 characters | spec/assurance/AD-004-checked-package-seam.md:174-175 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | dbf1475: Q-8 now says "Each of the five ADR-002 members FR-344 names as unadmitted (supertype list, abstractness flag, subsets edge, redefines edge, population node)" and says the other two are carried and admitted |
| FND-002 | fixed | dbf1475: each of Q-1..Q-9 is now marked current, stated or gap. Q-1 and Q-6 are gaps with the measured reason. Q-2 and Q-3 are current (FR-038 Description, FR-038-AC-4). Q-4 and Q-5 are stated, with no test measured. Q-8 is current (TC-222 ✅) |
| FND-003 | fixed | dbf1475: Q-7 now cites FR-037-AC-6 (planned) for replay and witness types and FR-028 for the model crate's QSL dependency, and says FR-037-AC-6 covers neither a terminal type nor a dependency rule |
| FND-004 | fixed | dbf1475: the AD now says "before any version-specific decode" and that the strict parse, duplicate-member check and canonical-bytes check run first (`read_value`) |
| FND-005 | fixed | dbf1475: the duplicate and unknown member row now gives the member's pointer. A `missing_import` row was added; verified at `dependency_references.rs:103` and `model_members.rs:866` |
| FND-006 | fixed | dbf1475: the reference reader is now named as QSpec's checked-package test harness, and rule 2 is worded as IR's policy, "not a ruling on QSpec's authority" |
| FND-007 | fixed | dbf1475: text admission is now recorded as a codegen term, verified at CG `spec/oracle/matrix/tests.md:14,22`. It is pending with the owner and not routed. R-S8 is removed from the QSpec table and Open questions is consistent |
| FND-008 | fixed | dbf1475: R-I1..R-I3 are now defined in a table. R-I1 is verified at `src/kani/outcome.rs:42` and `src/kani/mod.rs:23`; R-I3 at `src/lib.rs:12` |
