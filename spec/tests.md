---
id: TM-009
title: "quire-contract-ir test matrix index"
type: TestMatrixIndex
---
# TM-009: quire-contract-ir test matrix index

## Requirements Traceability

| Subsystem | Requirements | Local Matrix | Status |
|---|---|---|---|
| Core | StR-001, StR-002, StR-003, FR-011, FR-012, NFR-001, NFR-002, NFR-003, STD-001 | [core](./core/matrix/tests.md) | 🚧 StR-001, StR-003, NFR-001..NFR-003 and STD-001 are partial: cross-platform determinism and portability analysis (TC-019) and `kani_outcome_invalid` (TC-223) are planned |
| Model | FR-013, FR-014, FR-015, FR-016, FR-017, FR-019, FR-023, FR-028 | [model](./model/matrix/tests.md) | 🚧 FR-019-AC-5 (TC-058) and FR-028-AC-2 are planned; FR-028-AC-3's composition build is outside this repository |
| Conformance | FR-018, FR-020 | [conformance](./conformance/matrix/tests.md) | ✅ implemented |
| Checked package | FR-035, FR-038, FR-040, FR-344 | [checked_package](./checked_package/matrix/tests.md) | 🚧 FR-038's row is marked partial in its matrix (AC-46 through AC-61 planned, IR-530); AC-16 and AC-34 are not declared by FR-038; the other rows are implemented |
| Output mapping | FR-032, FR-033, FR-034, STD-003 | [output_mapping](./output_mapping/matrix/tests.md) | ✅ implemented |
| Kani | FR-029, FR-030, FR-031, FR-036, FR-037, FR-039 | [kani](./kani/matrix/tests.md) | 🚧 FR-029-AC-2, FR-030-AC-4, FR-030-AC-5, FR-036 (TC-045), FR-037-AC-6 and FR-039 (TC-055) planned |
