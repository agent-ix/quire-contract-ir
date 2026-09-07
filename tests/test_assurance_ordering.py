"""Record-before-attestation/receipt is an API boundary, not a debug assertion."""

import subprocess
import sys
import unittest
from pathlib import Path


class AssuranceOrderingTests(unittest.TestCase):
    def test_ordering_is_enforced_with_and_without_optimization(self) -> None:
        """TC-034. Trace: FR-022-AC-6."""
        probe = '''
from pathlib import Path
from unittest.mock import patch
from scripts.assurance_chain import Chain, ChainError

with patch("scripts.assurance_chain.observe_environment", return_value={}), \\
     patch("scripts.assurance_chain.quoin") as invoke:
    chain = Chain(Path("unused"), "candidate", {
        "record": {"definition": {"proof_obligations": []}}
    })
    operations = [
        (lambda: chain.seal_attestation(attestation_id="a", proof_id="p",
            output=Path("unused"), media_type="application/json", result="pass",
            tool_version="test"), "seal the record before an attestation"),
        (lambda: chain.receipt({}, Path("unused")), "seal the record before a receipt"),
    ]
    for operation, message in operations:
        try:
            operation()
        except ChainError as error:
            if str(error) != message:
                raise RuntimeError("wrong ordering error") from error
        else:
            raise RuntimeError("unsealed chain was accepted")
    invoke.assert_not_called()
'''
        for flags in ([], ["-O"]):
            with self.subTest(flags=flags):
                completed = subprocess.run(
                    [sys.executable, *flags, "-c", probe],
                    cwd=Path(__file__).resolve().parents[1],
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(completed.returncode, 0, completed.stderr)
