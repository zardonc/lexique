"""Exercise speech cache cleanup through a Windows desktop startup."""

import os
import ctypes
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import unittest


class SpeechCleanupTest(unittest.TestCase):
    def test_cleanup_waits_for_other_process_cache_lock(self):
        executable = os.environ.get("LEXIQUE_TEST_EXECUTABLE")
        if os.name != "nt" or not executable:
            self.skipTest("Windows release executable is required")
        import msvcrt
        with tempfile.TemporaryDirectory(prefix="lexique-speech-lock-") as temporary:
            root = Path(temporary)
            application = root / "Lexique.exe"
            shutil.copy2(executable, application)
            cache = root / "Lexique-data" / "temp" / "speech"
            cache.mkdir(parents=True)
            expired = cache / "1.ogg"
            expired.write_bytes(b"OggS test")
            old = time.time() - 8 * 24 * 60 * 60
            os.utime(expired, (old, old))
            lock = (cache / ".lock").open("w+b")
            lock.write(b"0")
            lock.flush()
            lock.seek(0)
            msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
            startup = subprocess.STARTUPINFO()
            startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            process = subprocess.Popen([str(application)], cwd=root,
                env=dict(os.environ, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--disable-gpu"), startupinfo=startup)
            try:
                database = root / "Lexique-data" / "data" / "lexique.sqlite3"
                deadline = time.monotonic() + 15
                while not database.exists() and process.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.1)
                self.assertTrue(database.exists())
                self.assertTrue(expired.exists(), "cleanup must wait for the other process")
                lock.seek(0)
                msvcrt.locking(lock.fileno(), msvcrt.LK_UNLCK, 1)
                deadline = time.monotonic() + 15
                while expired.exists() and process.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.1)
                self.assertFalse(expired.exists(), "cleanup should resume after lock release")
            finally:
                lock.close()
                subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                               check=False, capture_output=True)
                process.wait(timeout=15)
                time.sleep(1)

    def test_cleanup_failure_does_not_block_start_and_is_retried_on_restart(self):
        executable = os.environ.get("LEXIQUE_TEST_EXECUTABLE")
        if os.name != "nt" or not executable:
            self.skipTest("Windows release executable is required")
        with tempfile.TemporaryDirectory(prefix="lexique-speech-retry-") as temporary:
            root = Path(temporary)
            application = root / "Lexique.exe"
            shutil.copy2(executable, application)
            cache = root / "Lexique-data" / "temp" / "speech"
            cache.parent.mkdir(parents=True)
            cache.write_text("not a directory")
            database = root / "Lexique-data" / "data" / "lexique.sqlite3"
            startup = subprocess.STARTUPINFO()
            startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            environment = dict(os.environ, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--disable-gpu")
            log = root / "Lexique-data" / "logs" / "speech-cache.log"

            def start():
                process = subprocess.Popen([str(application)], cwd=root, env=environment,
                                           startupinfo=startup, stderr=subprocess.DEVNULL)
                return process

            def stop(process):
                subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                               check=False, capture_output=True)
                process.wait(timeout=15)
                time.sleep(1)

            process = start()
            try:
                deadline = time.monotonic() + 15
                while (not database.exists() or not log.exists() or b"Could not clean speech cache" not in log.read_bytes()) \
                        and process.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.1)
                self.assertIsNone(process.poll(), "cleanup failure must not close the app")
                self.assertTrue(database.exists(), "learning database should initialize")
                self.assertTrue(log.is_file(), "cleanup failures should persist without a console")
                self.assertIn(b"Could not clean speech cache", log.read_bytes())
            finally:
                stop(process)

            cache.unlink()
            cache.mkdir()
            expired = cache / "1.ogg"
            expired.write_bytes(b"OggS test")
            old = time.time() - 8 * 24 * 60 * 60
            os.utime(expired, (old, old))
            process = start()
            try:
                deadline = time.monotonic() + 15
                while expired.exists() and process.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.1)
                self.assertIsNone(process.poll(), "desktop app exited during retry")
                self.assertFalse(expired.exists(), "restart should retry cleanup")
            finally:
                stop(process)

    def test_startup_removes_only_expired_managed_audio(self):
        executable = os.environ.get("LEXIQUE_TEST_EXECUTABLE")
        if os.name != "nt" or not executable:
            self.skipTest("Windows release executable is required")
        with tempfile.TemporaryDirectory(prefix="lexique-speech-cleanup-") as temporary:
            root = Path(temporary)
            application = root / "Lexique.exe"
            shutil.copy2(executable, application)
            cache = root / "Lexique-data" / "temp" / "speech"
            cache.mkdir(parents=True)
            expired = cache / "1.ogg"
            recent = cache / "2.ogg"
            in_use = cache / "3.ogg"
            unrelated = cache / "notes.txt"
            abandoned = cache / ".4-123-456.tmp"
            temporary_recent = cache / ".5-123-457.tmp"
            unmanaged_temporary = cache / ".06-123-458.tmp"
            for file in (expired, recent, in_use, unrelated, abandoned, temporary_recent, unmanaged_temporary):
                file.write_bytes(b"OggS test")
            old = time.time() - 8 * 24 * 60 * 60
            os.utime(expired, (old, old))
            os.utime(in_use, (old, old))
            os.utime(abandoned, (old, old))
            os.utime(unmanaged_temporary, (old, old))
            kernel = ctypes.WinDLL("kernel32", use_last_error=True)
            kernel.CreateFileW.restype = ctypes.c_void_p
            kernel.CreateFileW.argtypes = [ctypes.c_wchar_p, ctypes.c_uint32, ctypes.c_uint32,
                                          ctypes.c_void_p, ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p]
            kernel.CloseHandle.argtypes = [ctypes.c_void_p]
            handle = kernel.CreateFileW(str(in_use), 0x80000000, 3, None, 3, 0, None)
            self.assertNotEqual(handle, ctypes.c_void_p(-1).value)
            startup = subprocess.STARTUPINFO()
            startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            environment = dict(os.environ, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--disable-gpu")
            process = subprocess.Popen([str(application)], cwd=root, env=environment, startupinfo=startup)
            try:
                deadline = time.monotonic() + 15
                while expired.exists() and process.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.1)
                self.assertIsNone(process.poll(), "desktop app exited during cleanup")
                self.assertFalse(expired.exists(), "expired audio should be removed")
                self.assertFalse(abandoned.exists(), "abandoned app-owned temporary audio should expire")
                self.assertTrue(recent.exists(), "recent audio should remain")
                self.assertTrue(temporary_recent.exists(), "recent temporary audio should remain")
                self.assertTrue(unmanaged_temporary.exists(), "noncanonical temporary names should remain")
                self.assertTrue(in_use.exists(), "audio in use should remain")
                self.assertTrue(unrelated.exists(), "unmanaged files should remain")
            finally:
                kernel.CloseHandle(handle)
                subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                               check=False, capture_output=True)
                process.wait(timeout=15)
                time.sleep(1)

    def test_startup_does_not_follow_speech_directory_link(self):
        executable = os.environ.get("LEXIQUE_TEST_EXECUTABLE")
        if os.name != "nt" or not executable:
            self.skipTest("Windows release executable is required")
        with tempfile.TemporaryDirectory(prefix="lexique-speech-link-") as temporary:
            root = Path(temporary)
            application = root / "Lexique.exe"
            shutil.copy2(executable, application)
            target = root / "other-data"
            target.mkdir()
            protected = target / "1.ogg"
            protected.write_bytes(b"OggS test")
            old = time.time() - 8 * 24 * 60 * 60
            os.utime(protected, (old, old))
            parent = root / "Lexique-data" / "temp"
            parent.mkdir(parents=True)
            try:
                (parent / "speech").symlink_to(target, target_is_directory=True)
            except OSError:
                result = subprocess.run(["cmd", "/c", "mklink", "/J", str(parent / "speech"), str(target)],
                                        capture_output=True)
                if result.returncode:
                    self.skipTest("Creating a directory reparse point is unavailable")
            startup = subprocess.STARTUPINFO()
            startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            environment = dict(os.environ, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--disable-gpu")
            process = subprocess.Popen([str(application)], cwd=root, env=environment, startupinfo=startup)
            try:
                database = root / "Lexique-data" / "data" / "lexique.sqlite3"
                deadline = time.monotonic() + 15
                while not database.exists() and process.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.1)
                self.assertTrue(database.exists(), "desktop app should initialize")
                time.sleep(1)
                self.assertTrue(protected.exists(), "linked directory must not be cleaned")
            finally:
                subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                               check=False, capture_output=True)
                process.wait(timeout=15)
                time.sleep(1)


if __name__ == "__main__":
    unittest.main()
