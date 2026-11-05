"""Output preservation and process budgets for the #25 resource experiments."""
import io
import json
from contextlib import redirect_stdout
from pathlib import Path
import tempfile
import sys
import time
import unittest
from unittest.mock import patch

import measure_s0
import measure_s0_resources as resources


class ResourceOutputTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.repo = Path(self.temporary.name)
        self.addCleanup(patch.stopall)
        patch.object(measure_s0, "REPO", self.repo).start()
        patch.object(resources, "REPO", self.repo).start()
        self.owner = self.repo / "target/issue-25"
        self.owner.mkdir(parents=True)
        self.space = {"create_model": lambda *args: object(), "generate": lambda *args:
            {"encoded": b"OggScontrolled-output", "pcm": b"controlled-pcm"}}
        patch.object(resources, "namespace", return_value=self.space).start()

    def invoke(self, output, mode):
        with patch("sys.argv", ["measure_s0_resources", str(self.repo / "resources"),
                str(output), *mode]), redirect_stdout(io.StringIO()):
            resources.main()

    def existing_output(self, output):
        output.mkdir(parents=True)
        for suffix in ("ogg", "f32"):
            (output / f"0.{suffix}").write_bytes(b"keep-existing-artifact")
        return output

    def assert_preserved(self, output):
        for suffix in ("ogg", "f32"):
            self.assertEqual((output / f"0.{suffix}").read_bytes(), b"keep-existing-artifact")

    def test_public_modes_reject_output_outside_owned_root_before_synthesis(self):
        for mode in (["--large"], ["--child", "0"]):
            with self.subTest(mode=mode):
                output = self.existing_output(self.repo / f"outside-{mode[0][2:]}")
                with patch.object(resources, "trial", side_effect=lambda *args:
                        resources.large_audio(args[0], args[1])):
                    with self.assertRaisesRegex(ValueError, "new child of target/issue-25"):
                        self.invoke(output, mode)
                self.assert_preserved(output)

    def test_public_modes_reject_existing_owned_outputs_before_synthesis(self):
        for mode in (["--large"], ["--child", "0"]):
            with self.subTest(mode=mode):
                output = self.existing_output(self.owner / f"existing-{mode[0][2:]}")
                with patch.object(resources, "trial", side_effect=lambda *args:
                        resources.large_audio(args[0], args[1])):
                    with self.assertRaises(FileExistsError):
                        self.invoke(output, mode)
                self.assert_preserved(output)

    def test_direct_modes_enforce_the_hard_process_budget_and_retain_timeout(self):
        for mode in (["--large"], ["--child", "0"]):
            with self.subTest(mode=mode):
                output = self.owner / f"timeout-{mode[0][2:]}"
                with patch.object(resources, "execute", return_value={"status": "timeout",
                        "ms": 60000, "error": "hard execution limit"}) as execute, \
                        patch.object(resources, "trial", return_value={}), \
                        patch.object(resources, "large_audio", return_value={}):
                    with self.assertRaises(SystemExit) as error:
                        self.invoke(output, mode)
                self.assertEqual(error.exception.code, 1)
                execute.assert_called_once()
                self.assertEqual(execute.call_args.kwargs["timeout"], 60)
                self.assertEqual(json.loads((output / "results.json").read_text())["status"], "timeout")

    def test_synthesis_requires_new_owned_outputs_even_when_called_internally(self):
        for background in (None, 0):
            with self.subTest(background=background):
                output = self.owner / f"synthesis-{background}"
                request = {"resources": str(self.repo / "resources"), "output": str(output),
                    "background": background}
                with patch.object(resources, "trial", side_effect=lambda *args:
                        resources.large_audio(args[0], args[1])):
                    result = resources.synthesize(request)
                    self.assertEqual(result["audio_bytes"], len(b"OggScontrolled-output"))
                    self.assertEqual((output / "0.ogg").read_bytes(), b"OggScontrolled-output")
                    self.assertEqual((output / "0.f32").read_bytes(), b"controlled-pcm")
                    with self.assertRaises(FileExistsError):
                        resources.synthesize(request)
                self.assertEqual((output / "0.ogg").read_bytes(), b"OggScontrolled-output")
                outside = self.existing_output(self.repo / f"internal-outside-{background}")
                with self.assertRaisesRegex(ValueError, "new child of target/issue-25"):
                    resources.synthesize({**request, "output": str(outside)})
                self.assert_preserved(outside)

    def test_direct_success_uses_a_real_subprocess_and_fresh_audio_files(self):
        output = self.owner / "subprocess-success"
        module_dir = str(Path(resources.__file__).parent)
        code = f"import sys,json;sys.path.insert(0,{module_dir!r});import measure_s0,measure_s0_resources as m;"
        code += f"measure_s0.REPO=__import__('pathlib').Path({str(self.repo)!r});"
        code += "m.namespace=lambda:{'create_model':lambda *a:object(),'generate':lambda *a:{'encoded':b'OggScontrolled-output','pcm':b'controlled-pcm'}};"
        code += "print(json.dumps(m.synthesize(json.load(sys.stdin))))"

        def controlled_execute(command, log, **kwargs):
            self.assertEqual(kwargs["timeout"], 60)
            return measure_s0.execute([sys.executable, "-I", "-B", "-X", "utf8", "-c", code], log, **kwargs)

        with patch.object(resources, "execute", side_effect=controlled_execute):
            self.invoke(output, ["--large"])
        report = json.loads((output / "results.json").read_text())
        self.assertEqual(report["status"], "passed")
        self.assertEqual(report["audio_directory"], "audio")
        self.assertEqual((output / "audio/0.ogg").read_bytes(), b"OggScontrolled-output")
        self.assertEqual((output / "audio/0.f32").read_bytes(), b"controlled-pcm")

    @unittest.skipUnless(sys.platform == "win32", "Windows resource process-tree cleanup")
    def test_direct_timeout_stops_its_synthesis_process_tree(self):
        output = self.owner / "subprocess-timeout"
        marker = self.repo / "unexpected-late-output"
        descendant = f"import time,pathlib;time.sleep(1.2);pathlib.Path({str(marker)!r}).write_text('still running')"
        code = f"import subprocess,sys,time;subprocess.Popen([sys.executable,'-c',{descendant!r}]);time.sleep(30)"

        def short_execute(command, log, **kwargs):
            self.assertEqual(kwargs.pop("timeout"), 60)
            return measure_s0.execute([sys.executable, "-I", "-B", "-c", code], log, timeout=0.5, **kwargs)

        with patch.object(resources, "execute", side_effect=short_execute):
            with self.assertRaises(SystemExit) as error:
                self.invoke(output, ["--large"])
        self.assertEqual(error.exception.code, 1)
        self.assertEqual(json.loads((output / "results.json").read_text())["status"], "timeout")
        time.sleep(1.4)
        self.assertFalse(marker.exists())

    def test_matrix_without_a_successful_solo_baseline_retains_all_failures(self):
        budgets = self.repo / "docs/evidence/issue-25/budgets.json"
        budgets.parent.mkdir(parents=True)
        budgets.write_text(json.dumps({"model_only_interaction": {}}))
        output = self.owner / "failed-matrix"
        with patch.object(resources, "execute", return_value={"status": "failed", "exit_code": 1, "ms": 1}):
            with self.assertRaises(SystemExit) as error:
                self.invoke(output, [])
        self.assertEqual(error.exception.code, 1)
        report = json.loads((output / "results.json").read_text())
        self.assertEqual(len(report["trials"]), 12)
        self.assertTrue(all(item["status"] == "failed" for item in report["trials"]))
        self.assertTrue(all(item["accepted"] is False for item in report["comparisons"]))
        self.assertEqual(report["large_audio"]["status"], "failed")


if __name__ == "__main__":
    unittest.main()
