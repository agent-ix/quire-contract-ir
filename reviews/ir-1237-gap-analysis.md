---
id: SR-202611010121
title: "IR-1237 PR 803 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 803; FR-038-AC-390..395, FR-019-AC-18 and FR-051-AC-6 trace and tests"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1237. Compared the computed Quire matrix, trace tags, public writer and strict-reader tests, and changed source. Plan completion: not assessed. No applicable AssuranceProfile was found.

## Verdict

**FAIL** — FR-051-AC-6 is omitted from the computed matrix because a blank line separates it from the Acceptance Criteria table, and no test asserts its required two-row typed writer-to-reader round trip. The FR-038-AC-390..395 and FR-019-AC-18 tags are present.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A blank line before FR-051-AC-6 splits the Markdown table, so `quire matrix` mints AC-1 through AC-5 but omits AC-6; the strict trace gate cannot report its missing test. Keep AC-6 inside the table. | spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md:54 |
| FND-002 | medium | The new typed writer test encodes one row based on a generic graph node and does not strictly read it; the QSpec test writes only rows returned by an already admitted package. Neither exercises an external producer constructing two typed PC-01/02 rows, encoding them, and strictly reading both as FR-051-AC-6 requires. | tests/it/checked_package_v2_term_data_writer.rs:230, tests/conformance_qspec/probabilistic_claims.rs:199 |

## Coverage

- FR-038-AC-390, role: examined. Excerpt: A present `CheckedTermDataV2` has exactly four required arrays, `equality_decisions`, `protocol_locations`, `dispatch_tables` and `probabilistic_claims`, including when all are empty. `None` omits the sidecar, while `Some(empty)` retains it through typed write and strict read. Removing any required array or adding an unknown member refuses `malformed_wire` at the missing object or unknown member; the public and private wire paths agree.
- FR-038-AC-391, role: examined. Excerpt: The strict reader admits an IR-constructed complete V2 package containing QSpec `checked-term-data-vectors.json` positive `PC-01` and `PC-02` identities as two nonempty probabilistic-claim rows. It retains each exact local `claim`/`probabilistic_claim` declaration `NodeRef` and its own declared `claim` occurrence. Their equal display names and source spans do not merge them or determine identity; a probabilistic-claim declaration without its one required row is refused under QSpec FR-322-AC-297.
- FR-038-AC-392, role: examined. Excerpt: The reader checks QSpec `PC-S1` missing occurrence and `PC-S2` extra member as closed-schema `malformed_wire` refusals. For `PC-R1` foreign, `PC-R2` wrong-kind, `PC-R3` absent-local and a non-probabilistic claim declaration, it returns `invalid_package`/`invalid-value` at `/checked_term_data/probabilistic_claims/{i}/declaration`; for `PC-R6` mis-mapped occurrence or wrong occurrence role, it returns that code/cause at `/checked_term_data/probabilistic_claims/{i}/occurrence`. Each refusal has the exact published primary RFC 6901 locus and no admitted package.
- FR-038-AC-393, role: examined. Excerpt: QSpec `PC-R4`'s second row for one declaration with a different valid occurrence and `PC-R5`'s repeated declaration-occurrence key both refuse `invalid_package`/`conflicting-definition` at the later row. `PC-R7`'s reversal of two valid rows refuses `invalid_package`/`invalid-semantic-graph` at the second row. Structural declaration identity and occurrence determine order; display name, source span and graph-array position do not.
- FR-038-AC-394, role: examined. Excerpt: IR tests read QSpec's published `checked-term-data-vectors.json` by reference and construct a complete valid V2 package in the test from its `PC-01` and `PC-02` row identities and resolved-node context. They run the positive and probabilistic-claim adverse mutations against the public strict reader and compare each published typed code, cause and exact RFC 6901 locus. No upstream artifact or vector content is copied into this repository, and the vector file is not treated as a complete checked-package fixture. An exact measured sidecar-work limit admits the IR-constructed positive; one below its measured charge returns `resource_exhausted` at its final probabilistic-claim row without a partial typed sidecar view.
- FR-038-AC-395, role: examined. Excerpt: A package with a `claim`/`probabilistic_claim` graph node passes the closed `ClaimForm::ProbabilisticClaim` vocabulary and ordinary typed term-body checks, then graph identity, exact dependencies, declaration, owner, occurrence and source-map joins before its probabilistic sidecar row is checked. The node and complete body survive admitted read/write without a fabricated formula or identity. An unknown claim form, malformed term body or broken graph join retains its existing earlier-stage typed refusal rather than being accepted by a matching sidecar row.
- FR-019-AC-18, role: examined. Excerpt: The default-feature model crate exports `ClaimForm::ProbabilisticClaim` and the public `CheckedProbabilisticClaim` row with exact typed `declaration: CheckedNodeId` and `occurrence: CheckedOccurrence` members, usable in the fourth required `CheckedTermDataV2::probabilistic_claims` array and borrowed `encode_checked_package_v2` input. A public consumer constructs nonempty rows without importing a private wire type or passing display name or source span as identity, and reads the same typed values from a strictly admitted package. The existing public-item inventory includes the row type when implementation lands.
- FR-051-AC-6, role: examined. Excerpt: A default-feature external producer constructs two typed nonempty probabilistic-claim rows using QSpec `PC-01` and `PC-02` identities by reference, passes them through the borrowed public encoder, and the strict reader admits both with their exact distinct declaration `NodeRef` and claim occurrence. The bytes equal the same four-array private envelope's canonical bytes; no name/span inference, row dropping, sorting or pre-admission occurs on the writer path. `None` and `Some` with four empty arrays remain distinct through write/read round trips.
- Computed matrix: FR-038-AC-390..395 and FR-019-AC-18 tagged; FR-051-AC-6 absent from the criterion set.
- Plan completion: not assessed

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Head e0996a76 removes the blank line; computed matrix now mints FR-051-AC-6 and reports two tagged tests. |
| FND-002 | fixed | Head e0996a76 adds a typed two-row QSpec producer encode and strict read test, plus absent/present empty round trips. |
