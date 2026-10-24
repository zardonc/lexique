import { spawn } from 'node:child_process';

const TIMEOUT_MS = 60_000;
const IS_WINDOWS = process.platform === 'win32';
const child = spawn('cargo', ['test', '--locked', '-p', 'lexique-storage'], {
  stdio: 'inherit',
  detached: !IS_WINDOWS,
});
const timer = setTimeout(() => {
  console.error(`Backend tests exceeded the hard ${TIMEOUT_MS / 1000}-second limit.`);
  process.exitCode = 1;
  if (IS_WINDOWS) {
    const killer = spawn('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'inherit' });
    killer.on('error', (error) => { console.error(error); child.kill('SIGKILL'); });
    return;
  }
  process.kill(-child.pid, 'SIGKILL');
}, TIMEOUT_MS);
child.on('error', (error) => {
  clearTimeout(timer);
  console.error(error);
  process.exitCode = 1;
});
child.on('exit', (code) => {
  clearTimeout(timer);
  process.exitCode = process.exitCode || (code ?? 1);
});
