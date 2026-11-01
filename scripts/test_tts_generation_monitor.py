"""Check that failed Windows memory sampling cannot validate a TTS trial."""
import ctypes
import os
import subprocess
import sys
import threading
import unittest
from unittest.mock import patch

from measure_tts_generation import Monitor


def failed_memory(*_):
    ctypes.set_last_error(5)
    return 0


@unittest.skipUnless(os.name == "nt", "Windows process sampling")
class MonitorTests(unittest.TestCase):
    def setUp(self):
        self.monitor = Monitor()
        self.monitor.stop.set()
        self.monitor.thread.join()

    def tearDown(self):
        self.monitor.error = None
        self.monitor.close()

    def test_normal_sample_has_live_memory(self):
        sample = self.monitor.sample()
        self.assertGreater(sample["working_set_bytes"], 0)
        self.assertGreater(sample["private_bytes"], 0)

    def test_live_memory_failure_rejects_sample_with_original_error(self):
        count = len(self.monitor.samples)
        with patch.object(self.monitor.psapi, "GetProcessMemoryInfo", failed_memory):
            with self.assertRaises(OSError) as raised:
                self.monitor.sample()
        self.assertEqual(raised.exception.winerror, 5)
        self.assertEqual(len(self.monitor.samples), count)

    def test_exited_child_memory_failure_keeps_remaining_live_memory(self):
        child = subprocess.Popen([sys.executable, "-I", "-B", "-c", "import sys; sys.stdin.read()"],
                                 stdin=subprocess.PIPE, creationflags=subprocess.CREATE_NO_WINDOW)
        try:
            self.monitor.add(child.pid)
            handle = self.monitor.handles[child.pid]
            child.stdin.close()
            self.assertEqual(child.wait(timeout=5), 0)
            original = self.monitor.psapi.GetProcessMemoryInfo
            def memory_query(process_handle, *args):
                return failed_memory() if process_handle == handle else original(process_handle, *args)
            with patch.object(self.monitor.psapi, "GetProcessMemoryInfo", memory_query):
                sample = self.monitor.sample()
            self.assertGreater(sample["working_set_bytes"], 0)
            self.assertGreater(sample["private_bytes"], 0)
        finally:
            if not child.stdin.closed:
                child.stdin.close()
            child.wait(timeout=5)

    def test_exit_query_failure_preserves_both_errors(self):
        def failed_exit(*_):
            ctypes.set_last_error(6)
            return 0
        count = len(self.monitor.samples)
        with patch.object(self.monitor.psapi, "GetProcessMemoryInfo", failed_memory), \
                patch.object(self.monitor.kernel, "GetExitCodeProcess", failed_exit):
            with self.assertRaises(OSError) as raised:
                self.monitor.sample()
        self.assertEqual(raised.exception.winerror, 6)
        self.assertEqual(raised.exception.__cause__.winerror, 5)
        self.assertEqual(len(self.monitor.samples), count)

    def test_background_failure_invalidates_close_and_releases_handles(self):
        self.monitor.stop.clear()
        self.monitor.thread = threading.Thread(target=self.monitor.loop, daemon=True)
        with patch.object(self.monitor.psapi, "GetProcessMemoryInfo", failed_memory):
            self.monitor.thread.start()
            self.monitor.thread.join(timeout=1)
            self.assertFalse(self.monitor.thread.is_alive())
            error = self.monitor.error
            with self.assertRaises(OSError) as raised:
                self.monitor.close()
        self.assertIs(raised.exception, error)
        self.assertEqual(raised.exception.winerror, 5)
        self.assertEqual(self.monitor.handles, {})

    def test_final_sample_failure_releases_handles(self):
        with patch.object(self.monitor.psapi, "GetProcessMemoryInfo", failed_memory):
            with self.assertRaises(OSError):
                self.monitor.close()
        self.assertEqual(self.monitor.handles, {})


if __name__ == "__main__":
    unittest.main()
