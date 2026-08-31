import hashlib
import json
import unittest
from pathlib import Path

from jsonschema import Draft7Validator, FormatChecker


ROOT = Path(__file__).resolve().parent.parent
RECORD = ROOT / "evidence/corrections/COR-001-pr12-code-review.json"
CHECKSUM = ROOT / "evidence/corrections/COR-001-pr12-code-review.sha256"
SCHEMA = ROOT / "schemas/evidence-correction-v1.schema.json"
MANIFEST = ROOT / "corpus/evidence-corrections/manifest.json"


class EvidenceCorrectionTests(unittest.TestCase):
    def test_correction_is_schema_valid_and_checksum_authenticated(self) -> None:
        schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
        record_bytes = RECORD.read_bytes()
        record = json.loads(record_bytes)

        Draft7Validator.check_schema(schema)
        errors = list(
            Draft7Validator(schema, format_checker=FormatChecker()).iter_errors(record)
        )
        self.assertEqual(errors, [])

        expected, relative = CHECKSUM.read_text(encoding="utf-8").strip().split("  ")
        self.assertEqual(relative, RECORD.relative_to(ROOT).as_posix())
        self.assertEqual(hashlib.sha256(record_bytes).hexdigest(), expected)

    def test_correction_corpus_matches_declared_validity(self) -> None:
        schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        validator = Draft7Validator(schema, format_checker=FormatChecker())

        self.assertEqual(manifest["schema"], SCHEMA.relative_to(ROOT).as_posix())
        declared = {fixture["path"] for fixture in manifest["fixtures"]}
        available = {RECORD.relative_to(ROOT).as_posix()}
        available.update(
            path.relative_to(ROOT).as_posix()
            for path in (ROOT / "corpus/evidence-corrections/invalid").glob("*.json")
        )
        self.assertEqual(declared, available)
        for fixture in manifest["fixtures"]:
            payload = json.loads((ROOT / fixture["path"]).read_text(encoding="utf-8"))
            self.assertEqual(not bool(list(validator.iter_errors(payload))), fixture["valid"])


if __name__ == "__main__":
    unittest.main()
