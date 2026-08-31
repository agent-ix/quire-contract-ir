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
