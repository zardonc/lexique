"""Run the complete Windows Sherpa checks or resource-free CI checks."""

import argparse
import os
from pathlib import Path
import sys
import unittest


ASSET_VARIABLES = (
    "LEXIQUE_SHERPA_ASSETS",
    "LEXIQUE_SHERPA_ASR_ASSETS",
    "LEXIQUE_SHERPA_SOURCE_DOWNLOADS",
    "LEXIQUE_SHERPA_MODEL_SOURCE",
)
SMOKE_TESTS = (
    "test_sherpa_probe.SherpaProbeTests.test_missing_resource_is_reported_without_creating_output",
    "test_sherpa_probe.SherpaProbeTests.test_existing_output_is_preserved",
    "test_sherpa_probe.SherpaProbeTests.test_corrupt_resource_fails_before_creating_output",
    "test_sherpa_sources.SherpaSourcesTests.test_missing_source_is_reported_before_creating_delivery",
    "test_sherpa_sources.SherpaSourcesTests.test_existing_directory_zip_or_measurements_are_preserved",
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--smoke", action="store_true", help="run five checks without model assets or playback")
    args = parser.parse_args()
    if not args.smoke:
        missing = [name for name in ASSET_VARIABLES if not os.environ.get(name)]
        if missing:
            parser.exit(2, f"Missing required asset variables: {', '.join(missing)}\n")
        if sys.platform != "win32":
            parser.exit(2, "Complete Sherpa validation requires a network-denied Windows process.\n")

    sys.path.insert(0, str(Path(__file__).resolve().parent))
    names = SMOKE_TESTS if args.smoke else ("test_sherpa_probe", "test_sherpa_sources")
    suite = unittest.defaultTestLoader.loadTestsFromNames(names)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if result.skipped:
        print("Selected Sherpa checks were skipped; validation is incomplete.", file=sys.stderr)
    return 0 if result.wasSuccessful() and not result.skipped else 1


if __name__ == "__main__":
    raise SystemExit(main())
