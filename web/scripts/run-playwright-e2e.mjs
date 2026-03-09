import { spawn } from 'node:child_process';

const HOST = '127.0.0.1';
const PREVIEW_START_TIMEOUT_MS = 240_000;
const SHUTDOWN_TIMEOUT_MS = 5_000;
const BASE_URL_REGEX = new RegExp(`http://${HOST.replaceAll('.', '\\.')}:\\d+`);

const npmCmd = process.platform === 'win32' ? 'npm.cmd' : 'npm';
const npxCmd = process.platform === 'win32' ? 'npx.cmd' : 'npx';

let previewProcess = null;
let playwrightProcess = null;

function stripAnsi(text) {
  return text.replace(/\u001b\[[0-?]*[ -/]*[@-~]/g, '');
}

function extractBaseUrl(text) {
  const normalized = stripAnsi(text);
  const match = normalized.match(BASE_URL_REGEX);
  return match ? match[0] : null;
}

function waitForExit(child) {
  return new Promise(resolve => {
    if (child.exitCode !== null || child.signalCode !== null) {
      resolve();
      return;
    }
    child.once('exit', () => resolve());
  });
}

async function stopProcess(child) {
  if (!child || child.exitCode !== null || child.signalCode !== null) {
    return;
  }

  child.kill('SIGTERM');
  const exited = await Promise.race([
    waitForExit(child).then(() => true),
    new Promise(resolve => setTimeout(() => resolve(false), SHUTDOWN_TIMEOUT_MS))
  ]);

  if (!exited) {
    child.kill('SIGKILL');
    await waitForExit(child);
  }
}

function startPreviewServer() {
  return new Promise((resolve, reject) => {
    previewProcess = spawn(npmCmd, ['run', 'preview:e2e', '--', '--port', '0'], {
      stdio: ['ignore', 'pipe', 'pipe'],
      env: process.env
    });

    let combinedOutput = '';
    let settled = false;
    const startupTimer = setTimeout(() => {
      if (settled) {
        return;
      }
      settled = true;
      reject(new Error(`Timed out after ${PREVIEW_START_TIMEOUT_MS}ms while waiting for preview server.`));
    }, PREVIEW_START_TIMEOUT_MS);

    const onChunk = chunk => {
      const text = chunk.toString();
      process.stderr.write(text);
      combinedOutput = `${combinedOutput}${text}`.slice(-8192);
      if (settled) {
        return;
      }
      const baseUrl = extractBaseUrl(combinedOutput);
      if (!baseUrl) {
        return;
      }
      settled = true;
      clearTimeout(startupTimer);
      resolve(baseUrl);
    };

    previewProcess.stdout.on('data', onChunk);
    previewProcess.stderr.on('data', onChunk);

    previewProcess.once('error', error => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(startupTimer);
      reject(error);
    });

    previewProcess.once('exit', (code, signal) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(startupTimer);
      reject(new Error(`Preview server exited before ready (code=${code ?? 'null'}, signal=${signal ?? 'null'}).`));
    });
  });
}

function runPlaywright(baseUrl) {
  return new Promise((resolve, reject) => {
    playwrightProcess = spawn(npxCmd, ['playwright', 'test', ...process.argv.slice(2)], {
      stdio: 'inherit',
      env: {
        ...process.env,
        PW_E2E_BASE_URL: baseUrl
      }
    });

    playwrightProcess.once('error', error => {
      reject(error);
    });

    playwrightProcess.once('exit', code => {
      playwrightProcess = null;
      resolve(code ?? 1);
    });
  });
}

async function shutdown(exitCode) {
  await stopProcess(playwrightProcess);
  await stopProcess(previewProcess);
  process.exit(exitCode);
}

process.on('SIGINT', () => {
  void shutdown(130);
});

process.on('SIGTERM', () => {
  void shutdown(143);
});

try {
  const baseUrl = await startPreviewServer();
  console.error(`[E2E] Using preview server: ${baseUrl}`);
  const exitCode = await runPlaywright(baseUrl);
  await shutdown(exitCode);
} catch (error) {
  console.error(error);
  await shutdown(1);
}
