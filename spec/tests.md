---
id: TM-009
title: "quire-contract-ir test matrix index"
type: TestMatrixIndex
---
# TM-009: quire-contract-ir test matrix index

## Requirements Traceability

| Subsystem | Requirements | Local Matrix | Status |
|---|---|---|---|
| Core | StR-001, StR-002, StR-003, FR-011, FR-012, NFR-001, NFR-002, NFR-003, STD-001 | [core](./core/matrix/tests.md) | 🚧 cross-platform determinism and portability analysis (TC-019) planned |
| Model | FR-013, FR-014, FR-015, FR-016, FR-017, FR-019, FR-023, FR-028 | [model](./model/matrix/tests.md) | 🚧 FR-019-AC-5 (TC-058), FR-028-AC-2 and FR-028-AC-3's composition build planned |
| Conformance | FR-018, FR-020 | [conformance](./conformance/matrix/tests.md) | ✅ implemented |
| Checked package | FR-035, FR-036, FR-038, FR-040, FR-344 | [checked_package](./checked_package/matrix/tests.md) | 🚧 FR-036 provider negotiation (TC-045) planned |
| Output mapping | FR-032, FR-033, FR-034, STD-003 | [output_mapping](./output_mapping/matrix/tests.md) | ✅ implemented |
| Kani | FR-029, FR-030, FR-031 | [kani](./kani/matrix/tests.md) | 🚧 FR-029-AC-2, FR-030-AC-4 and FR-030-AC-5 planned |
| Bridge | FR-037, FR-039 | [bridge](./bridge/matrix/tests.md) | 🚧 root crate public interface (TC-055) planned |
