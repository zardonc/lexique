"""Check the strict and resource-free Sherpa validation commands."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("check_sherpa.py")
ASSET_VARIABLES = (
    "LEXIQUE_SHERPA_ASSETS",
    "LEXIQUE_SHERPA_ASR_ASSETS",
    "LEXIQUE_SHERPA_SOURCE_DOWNLOADS",
    "LEXIQUE_SHERPA_MODEL_SOURCE",
)


class SherpaCommandTests(unittest.TestCase):
    def run_check(self, directory, *options, supplied_assets=False):
        environment = os.environ.copy()
        for name in ASSET_VARIABLES:
            environment.pop(name, None)
            if supplied_assets:
                environment[name] = str(Path(directory) / "missing-assets")
        return subprocess.run(
            [sys.executable, "-B", "-X", "utf8", str(SCRIPT), *options],
            cwd=directory, env=environment,
            capture_output=True, text=True, encoding="utf-8", timeout=60,
        )

    def test_full_validation_requires_all_asset_variables(self):
        with tempfile.TemporaryDirectory() as directory:
            result = self.run_check(directory)
            self.assertEqual(result.returncode, 2, result.stderr)
            for name in ASSET_VARIABLES:
                self.assertIn(name, result.stderr)
            self.assertNotIn("Ran ", result.stderr)

    def test_smoke_checks_require_no_assets_even_when_variables_are_set(self):
        with tempfile.TemporaryDirectory() as directory:
            for supplied in (False, True):
                with self.subTest(supplied_assets=supplied):
                    result = self.run_check(directory, "--smoke", supplied_assets=supplied)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.assertIn("Ran 5 tests", result.stderr)
                    self.assertNotIn("skipped", result.stderr)


if __name__ == "__main__":
    unittest.main()
