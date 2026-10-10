---
id: TM-004
title: "quire-contract-ir model test matrix"
type: TestMatrix
---
# quire-contract-ir model test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-013 | FR-013-AC-1 through FR-013-AC-5 | TC-016, TC-018 | ✅ implemented: AC-1 through AC-4 (TC-016) and AC-5 (IR-274 code change B), the decimal-string spelling of the eight v1 integer members and its refusals, verified in the unit tests of `wire.rs` (every member under every number and out-of-grammar spelling of AC-5 as `invalid_wire_format` from the member's own type, the out-of-range strings as `invalid_numeric_bounds`, and the widest strings decoded and serialized back to themselves) and by the published fixture schema and corpus (TC-018) |
| FR-014 | FR-014-AC-1 through FR-014-AC-9 | TC-016, TC-444 | 🚧 AC-1 through AC-8 implemented; AC-8 is tagged by `wide_public_numeric_values_retain_their_type_and_literal_payloads` in `tests/it/expression.rs`. AC-9's private numeric type/range construction and signature inspection is planned (TC-444); the current independent `Checked.value_type` and `Checked.range` fields do not satisfy it. |
| FR-015 | FR-015-AC-1 through FR-015-AC-7 | TC-016 | ✅ implemented |
| FR-016 | FR-016-AC-1 through FR-016-AC-8 | TC-017 | ✅ implemented: AC-1 through AC-4 including the caller byte budget on all five closed object kinds, with AC-1's golden fixtures re-recorded from `quire-canonical`'s output, and AC-5 through AC-8 (IR-274 code change B): the eight integer members as decimal strings, every byte and digest from `quire-canonical` (the crate's `CanonicalWriter` and `canonical_envelope_bytes` are deleted), no `null`, float or `Value` in the semantic value and revisions and byte offsets bounded at 2^53 by construction, and the explicit domain-prefixed digest. The canonical digest of every object holding one of the eight members changed once, with this code change |
| FR-017 | FR-017-AC-1 through FR-017-AC-6 | TC-017 | 🚧 AC-1 and AC-2 have existing TC-017 evidence; AC-3 through AC-6 are specified and await direct row-order, deep-precedence, duplicate-collapse, and authored-diagnostic-order assertions |
| FR-019 | FR-019-AC-1 through FR-019-AC-7 | TC-018, TC-058 | 🚧 AC-1 through AC-4, AC-6 and AC-7 have TC-018 evidence; AC-5 is verified by TC-058: the model root names exactly FR-019's public items with no glob, the fault-injection feature adds only `MappingAllocationPoint`, and positive/negative path probes show a model item is unavailable through `quire_contract_ir`. AC-6's artifact-reference member sets and absent `CheckedRevision` remain verified by `tc_018_the_artifact_reference_member_sets_are_exact` and model-root compile-fail doctests. |
| FR-023 | FR-023-AC-1 through FR-023-AC-5 | TC-035 | ✅ implemented |
| FR-028 | FR-028-AC-1 through FR-028-AC-5 | TC-041 | 🚧 AC-1, AC-3 and AC-4 are backed by `tc_041_model_dependency_graph_is_cycle_free_and_owner_free` in `tests/it/cycle_free_model.rs`; AC-3 checks that the root manifest has no QSL-repository dependency by name or git source. AC-2 and AC-5 are unbacked. The deleted bridge re-export test did not verify them. QSL-side graph and cross-repo composition checks remain planned in agent-ix/quire-integration (Linear IR-358). |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-016 | Types, expressions, short-circuiting, and definedness conform | Integration | P0 | FR-012..FR-015, NFR-002, STD-001 | ✅ implemented |
| TC-017 | Canonical bytes, digests, version preflight, and orphan classes conform | Property | P0 | FR-016, FR-017, NFR-001, NFR-003 | 🚧 existing FR-016 and FR-017-AC-1/2 evidence implemented: FR-016-AC-1 through AC-4 in `tests/it/canonicalization.rs`, with its expected bytes written out and no number spelling of the eight members; FR-016-AC-5 through AC-8 in `tests/it/canonical_v1_quire_canonical.rs` (IR-274 code change B): the eight integer members at `i64::MIN`, `i64::MAX`, `0` and `-1` and `maximum_denominator` `i64::MAX` as strings, one fixture of each of the five kinds whose bytes equal an expected byte string written out in the test and whose digest equals the SHA-256 of the explicit domain prefix and those bytes (not `sha256_with_domain`, which the test also shows differs), a limit of the exact length and one byte lower, a source scan for `CanonicalWriter`, `canonical_envelope_bytes`, `digest_json` and `serde_json_canonicalizer` and for a `serde_json::to_vec` or `serde_json::to_value` call in `canonical.rs`, `binding.rs` and `output_mapping.rs`, and a scan of the canonical types for a float, a `Value` or an `Option` member; the 2^53 bound on a revision and byte offset is FR-011-AC-3's and FR-012-AC-6's (TC-015); direct assertions for FR-017-AC-3 through AC-6 and NFR-003-AC-3 remain planned. The current duplicate fixture asserts the later occurrence source span but does not assert exact reference-error spans |
| TC-035 | Derived executable projections bind complete typed clause populations through the public IR boundary | Integration | P0 | FR-023 | ✅ implemented |
| TC-041 | Reach the model through `quire_contract_model` while proving the model/owner/root Cargo graph is acyclic | Integration | P0 | FR-028-AC-1, FR-028-AC-2, FR-028-AC-3, FR-028-AC-4, FR-028-AC-5 | 🚧 AC-1, AC-3 and AC-4 are backed by `tc_041_model_dependency_graph_is_cycle_free_and_owner_free`; AC-2 and AC-5 remain unbacked. QSL-side graph and cross-repo composition checks remain planned in agent-ix/quire-integration (Linear IR-358). |
| TC-058 | The model crate's public interface is exactly FR-019's list, re-exported by name | Integration | P0 | FR-019-AC-5 | ✅ implemented: `tests/it/public_interface.rs` checks the no-glob and exact default inventory, sole feature addition, and root boundary; `src/lib.rs` doctests compile the model path and refuse the root path. Artifact-reference member-set probes belong to FR-019-AC-6 and TC-018. |
| TC-444 | Private numeric type and range are paired by construction | Inspection | P0 | FR-014-AC-9 | 🚧 planned: inspect every private checked-numeric constructor and unary and binary operator signature; current independent type/range fields fail the criterion |

## Coverage Design

| Test | Coverage rule | Required cases |
|---|---|---|
| TC-041 cases | Coverage, dependency, feature, edge | Cargo metadata cycle check for default/all/minimum features; model package owner/TL dependency absence; unchanged schema, canonical-output, diagnostic and corpus results; root package manifest names no `quire-spec-language` or `qsl-*` dependency (the QSL-side graph check and the locked root-package-and-QSL composition build live in agent-ix/quire-integration); compile-fail probes for public owner constructors, callbacks, trait validators, trust flags and copied owner wire types. TC-058 owns the model-versus-root import path probe. |
| TC-058 cases | Coverage, compile-fail, closure | no glob `pub use` in the model crate root; default-feature public-item inventory equal to FR-019's table both ways; `fault-injection` inventory adds `MappingAllocationPoint` alone; one table item builds through `quire_contract_model` and fails through `quire_contract_ir` |
| TC-018 cases (FR-019-AC-6) | Coverage, compile-fail | the member sets of `CheckedArtifactRef`, `CheckedSourceRef` and `CheckedArtifactLocator` build exactly (a struct literal naming every member) and fail with an added member, and `CheckedRevision` fails to compile |
