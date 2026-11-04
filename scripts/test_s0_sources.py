"""Reconstruct frozen #25 inputs without depending on uncommitted product code."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import measure_s0


class FrozenSourceTests(unittest.TestCase):
    def setUp(self):
        owner = measure_s0.REPO / "target/issue-25"
        owner.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(prefix="source-regression-", dir=owner)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        archive = self.root / "clean.zip"
        subprocess.run(["git", "archive", "--format=zip", "--output", str(archive),
            "3c48180dda8abd47009d86441c28647bb22d658f"], cwd=measure_s0.REPO,
            check=True, capture_output=True, timeout=60)
        self.clean = self.root / "clean"
        with zipfile.ZipFile(archive) as bundle:
            bundle.extractall(self.clean)
        self.evidence = self.clean / "docs/evidence/issue-25"
        for name in ("source-inputs.json", "frozen-source.patch"):
            candidate = measure_s0.REPO / "docs/evidence/issue-25" / name
            if candidate.exists():
                (self.evidence / name).write_bytes(candidate.read_bytes())
        self.output = self.clean / "target/issue-25/build"
        self.output.mkdir(parents=True)
        self.manifest = json.loads((self.evidence / "manifest.json").read_text())

    def generate(self):
        with patch.object(measure_s0, "REPO", self.clean):
            return measure_s0.sources(self.output)

    def test_clean_checkout_reconstructs_the_exact_frozen_observation_sources(self):
        self.assertNotIn("type AudioInspection =", (self.clean /
            "apps/desktop/src-tauri/src/vocabulary.rs").read_text())
        generated = self.generate()
        expected = self.manifest["worker_observer"]["generated_sha256"]
        self.assertEqual({name: hashlib.sha256(value.encode()).hexdigest()
            for name, value in generated.items()}, expected)
        source = self.output / "source"
        self.assertEqual(hashlib.sha256((source / "Cargo.lock").read_bytes()).hexdigest(),
            self.manifest["cargo_lock_sha256"])

    def test_unrelated_working_tree_changes_cannot_change_the_frozen_inputs(self):
        (self.clean / "apps/desktop/src-tauri/src/vocabulary.rs").write_text("unrelated work")
        (self.clean / "Cargo.lock").write_text("unrelated dependency changes")
        generated = self.generate()
        self.assertEqual(hashlib.sha256(generated["vocabulary.rs"].encode()).hexdigest(),
            self.manifest["worker_observer"]["generated_sha256"]["vocabulary.rs"])
        self.assertEqual((self.clean / "Cargo.lock").read_text(), "unrelated dependency changes")

    def test_modified_snapshot_is_rejected_before_restoration_or_compilation(self):
        with (self.evidence / "frozen-source.patch").open("ab") as file:
            file.write(b"corrupted snapshot")
        with self.assertRaisesRegex(ValueError, "Frozen source patch hash mismatch"):
            self.generate()
        self.assertFalse((self.output / "source").exists())

    def test_restoration_never_reuses_an_existing_source_directory(self):
        self.generate()
        with self.assertRaises(FileExistsError):
            self.generate()


if __name__ == "__main__":
    unittest.main()
