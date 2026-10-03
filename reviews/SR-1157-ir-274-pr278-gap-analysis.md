---
id: SR-1157
title: "gap analysis of PR 278 (IR-274 part B: v1 decimal-string wire and canonical objects through quire-canonical)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@297810203764f5cab680427ea79ff9da26956bf4; spec/model/functional/FR-013-type-system.md (AC-5), spec/model/functional/FR-016-canonicalization-digests.md (AC-1, AC-4 to AC-8), spec/conformance/functional/FR-020-json-conformance-interface.md (AC-3 and prose), spec/core/functional/FR-011-package-identity.md (AC-3), spec/core/functional/FR-012-anchors-clauses-dependencies.md (AC-6), spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-80, AC-91, AC-92), spec/core/functional/STD-001-diagnostic-registry.md, spec/model/matrix/tests.md, spec/conformance/matrix/tests.md, spec/core/matrix/tests.md, spec/checked_package/matrix/tests.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/output_mapping/matrix/tests.md, spec/tests.md, and the code and tests of SR-1156"
review_set: subset
---
# SR-1157: gap analysis of PR 278

## Summary

Ticket: IR-274 (code change B). Planless gap analysis of the ACs the PR
claims, against the code, the tagged tests and the matrix rows it edits.

Per AC:

- FR-013-AC-5: implemented. `wire.rs` unit tests run every one of the eight
  members under the five numbers and seven strings of the AC
  (`invalid_wire_format`), the out-of-range strings and `"0"` for
  `maximum_denominator` (`invalid_numeric_bounds`), and decode the widest
  strings and serialize them back. Tagged TC-016.
- FR-016-AC-1, AC-4: unchanged tests still pass on the re-recorded goldens; the
  rational golden in `canonicalization.rs` is now a full hand-written string.
- FR-016-AC-5: implemented: the cases of the AC are written out byte for byte,
  `maximum_items` and `revision` stay numbers, and the corpus scan finds no
  member followed by a digit or `-`.
- FR-016-AC-6: implemented: five kinds equal hand-written strings, digests
  equal the prefix hash of those strings, exact limit passes and one byte
  lower refuses `canonicalization_resource_exhausted`, and the symbol scan
  reads the three files and both manifests.
- FR-016-AC-7: implemented as the AC words it (a source scan of the named
  definitions, plus revision 2^53 canonicalizing as a number).
- FR-016-AC-8: implemented; the test shows the digest differs from
  `sha256_with_domain` with the same label and from bare `sha256`.
- FR-038-AC-91/92 for `canonical.rs` and `binding.rs`: implemented (TC-048
  scan; my grep agrees).
- FR-020-AC-3: partial, as the PR says (FND-001).
- FR-011-AC-3, FR-012-AC-6: the constructors and the package decoder refuse
  above 2^53 with the registered codes and admit 2^53 exactly; both schemas
  bound at 2^53 (tested at 2^53 and 2^53+1). The expression decode path does
  not use the registered codes (FND-002).

`document_json` is not an invention: the package operation already accepted
it on `main` (fixture schema line 397). The PR extends the same form to the
expression operation and names it in FR-020's prose; FR-020-AC-3 only
requires that a decoder fixture "supplies a JSON number straight to the
decoder", which this does.

Spec text edited by the PR: the FR-020-AC-3 status prefix (the row already
carried "Planned, landing with code change B") and FR-020 prose describing
the landed state. Matrix rows for FR-013, FR-016, TC-017, FR-011, FR-012,
TC-048 and the index are accurate except as FND-001 and FND-002 note.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-020-AC-3's last clause ("every canonical file ... whose object holds one of the eight members equals the expected bytes written in the test") is met for four of 47 files, as the PR body and the FR-020 row say honestly. But the row, the index and the AC prefix name no ticket for the missing clause, so nothing tracks it once IR-274 closes. Repo convention for a partial AC (FR-038-AC-110: "stays planned (IR-555)") names the clause as planned with a ticket. File one (or route a spec amendment if the clause is judged excessive) and cite it in the FR-020 row and `spec/tests.md`. For the record: this review compared all 47 files with their base bytes and each differs only by the eight members' respelling, so the gap is in the test, not the corpus | spec/conformance/matrix/tests.md:13; spec/tests.md:14; spec/conformance/functional/FR-020-json-conformance-interface.md:136 |
| FND-002 | low | Through the expression decoder (the expression operation and every executable-projection binding), a requirement revision or a source byte offset above 2^53 is refused as `invalid_wire_format` at `expression`, not `invalid_requirement_revision` or `invalid_source_span`, because `RequirementRef` and `SourceSpan` are deserialized by serde there and the constructor's diagnostic becomes a serde custom error. Measured with `document_json` probes in a scratch corpus (owner revision 9007199254740993, span end offset 9007199254740993: both `invalid_wire_format`). A zero revision behaves the same on `main`, so this is the existing pattern on that path, but the FR-011 and FR-012 rows now say ✅ with no scope note. Either map the constructor diagnostics through on that path or scope the rows to the package decoder | crates/quire-contract-model/src/wire.rs:21; crates/quire-contract-model/src/identity.rs:455-482; spec/core/matrix/tests.md:20-21 |

## Verdict

Changes requested on matrix tracking (FND-001); FND-002 can be scoped in the
same round. Every other claimed AC is implemented and backed by a tagged test
with a hand-written or independent oracle.

## Dispositions

Round 1, reviewed at e8d73a5a74888339dbe09274de4ddda9f50e3768. IR-568 and
IR-569 are Backlog sub-issues of IR-274, each stating its finding and what
closes it.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | deferred | IR-568 (child of IR-274): the FR-020 row (spec/conformance/matrix/tests.md:13), StR-002 in spec/core/matrix/tests.md and the Conformance line of spec/tests.md now say the clause stays partial and cite IR-568 (bc0b172); the AC text is unchanged and the PR body's Known gaps says the same. This follows the repo's FR-038-AC-110 / IR-555 convention |
| FND-002 | deferred | IR-569 (child of IR-274): mapping the constructor codes through the expression decoder means changing the serde field types of `ExpressionInput.owner` and every wire span, and a zero revision already gives `invalid_wire_format` on that path on main, so leaving the restructure to a follow-up is reasonable. The FR-011 and FR-012 rows went from ✅ to 🚧, limited to the constructors and the package decoder, and name the expression-decoder behaviour and IR-569 (e8d73a5). spec/tests.md's Core line and the PR body say the same, so the four places agree |
