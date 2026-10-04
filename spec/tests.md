---
id: TM-009
title: "quire-contract-ir test matrix index"
type: TestMatrixIndex
---
# TM-009: quire-contract-ir test matrix index

## Requirements Traceability

| Subsystem | Requirements | Local Matrix | Status |
|---|---|---|---|
| Core | StR-001, StR-002, StR-003, FR-011, FR-012, NFR-001, NFR-002, NFR-003, STD-001 | [core](./core/matrix/tests.md) | 🚧 StR-001, StR-003, NFR-001..NFR-003 and STD-001 are partial: cross-platform determinism and portability analysis (TC-019) and `kani_outcome_invalid` (TC-223) are planned; FR-011-AC-3 and FR-012-AC-6 (the 2^53 bound on a revision and a byte offset) give their registered codes through the constructors, the package decoder, the expression operation and the binding expression decoder, and are unmet on the whole-projection path, which is refused `invalid_wire_format` by the schema first, and on the coverage operation's artifact-trace spans, which are still serde-decoded (IR-574) |
| Model | FR-013, FR-014, FR-015, FR-016, FR-017, FR-019, FR-023, FR-028 | [model](./model/matrix/tests.md) | 🚧 FR-013-AC-5 and FR-016-AC-5 through FR-016-AC-8 are implemented (IR-274 code change B); FR-019-AC-5 (TC-058; the no-glob, inventory and path clauses; AC-6, the artifact-reference member sets, is implemented) and FR-028-AC-2 are planned; FR-028-AC-3's composition build is outside this repository |
| Conformance | FR-018, FR-020 | [conformance](./conformance/matrix/tests.md) | ✅ implemented (FR-020-AC-3, IR-274 code change B: every one of the 47 canonical files holding one of the eight integer members equals bytes written out in the test) |
| Checked package | FR-035, FR-038, FR-040, FR-344, FR-346 | [checked_package](./checked_package/matrix/tests.md) | 🚧 FR-038's row is marked partial in its matrix (AC-81 through AC-88 implemented, IR-532; AC-112 and AC-113 implemented, IR-552, run by `make conformance-qspec`; AC-114 through AC-118 implemented, IR-495 (AC-118 run by `make conformance-qspec`); AC-96 through AC-108 implemented, IR-549, and AC-66 retired; AC-119 through AC-122 implemented, IR-551; AC-109 through AC-111 implemented, IR-542 (AC-110's `1e400`/`-1e400` mapping, IR-555, and its first-fault rule, IR-573); AC-107's target is not wired into CI); AC-16 and AC-34 are not declared by FR-038; the other rows are implemented |
| Output mapping | FR-032, FR-033, FR-034, STD-003 | [output_mapping](./output_mapping/matrix/tests.md) | ✅ FR-032-AC-6, FR-033-AC-6, FR-034-AC-6 and FR-034-AC-7 through AC-9 (identity material under the request byte limit, decimal-string `u64`s; IR-274 code change C; IR-567 amended AC-7 to what admission does and split AC-8 and AC-9 out) are implemented |
| Kani | FR-029, FR-030, FR-031, FR-036, FR-037, FR-039 | [kani](./kani/matrix/tests.md) | 🚧 FR-029-AC-2, FR-030-AC-4, FR-030-AC-5, FR-036 (TC-045), FR-037-AC-6 and FR-039 (TC-055) planned |
