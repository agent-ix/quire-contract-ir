"""Tests for fail-closed matrix status classification."""

import unittest

from scripts.validate_matrix_status import validate_documents


class MatrixStatusTests(unittest.TestCase):
    def test_rejects_complete_rows_backed_by_planned_tests(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-3."""
        document = """
| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| NFR-001 | NFR-001-AC-1 | TC-019 | ✅ Complete |

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-019 | portability | Analysis | P0 | NFR-001 | 🚧 planned |
"""
        failures = validate_documents([document], set())
        self.assertEqual(failures, ["NFR-001 is complete but TC-019 is not complete"])

        no_test = """
| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| NFR-001 | NFR-001-AC-1 | none | ✅ Complete |
"""
        self.assertEqual(
            validate_documents([no_test], set()),
            ["NFR-001 is complete but references no test"],
        )

    def test_rejects_policy_acceptance_citation_without_executable_test(self) -> None:
        """TC-021. Trace: TC-021, NFR-004-AC-3."""
        matrix = """
| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-026 | dispositions | Inspection | P0 | FR-021 | ✅ implemented |
"""
        policy = """
| ID | Criterion | Verification |
|---|---|---|
| PGM-01-R11-AC-1 | dispositions are linked | TC-026 |
"""
        self.assertEqual(
            validate_documents([matrix, policy], set()),
            [
                "TC-026 is complete but has no executable test",
                "PGM-01-R11-AC-1 cites non-executable TC-026",
            ],
        )
        self.assertEqual(validate_documents([matrix, policy], {"TC-026"}), [])
