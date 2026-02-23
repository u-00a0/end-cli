import { defineConfig, loadEnv, type PluginOption, type ViteDevServer, normalizePath } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { spawn } from 'child_process';
import path from 'path';

function normalizeBasePath(rawBasePath: string | undefined): string {
  const trimmed = rawBasePath?.trim();
  if (!trimmed) {
    return '/';
  }

  const withLeadingSlash = trimmed.startsWith('/') ? trimmed : `/${trimmed}`;
  return withLeadingSlash.endsWith('/') ? withLeadingSlash : `${withLeadingSlash}/`;
}

function rustWasmDevPlugin(): PluginOption {
  return {
    name: 'rust-wasm-dev-rebuild',
    apply: 'serve',
    configureServer(server: ViteDevServer) {
      const webRoot = server.config.root;
      const repoRoot = path.resolve(webRoot, '..');

      const rustSrcDir = normalizePath(path.resolve(repoRoot, 'crates'));
      const rootCargoToml = normalizePath(path.resolve(repoRoot, 'Cargo.toml'));
      const rootCargoLock = normalizePath(path.resolve(repoRoot, 'Cargo.lock'));
      const buildScript = normalizePath(path.resolve(repoRoot, 'scripts/build_web_wasm.sh'));

      server.watcher.add([rustSrcDir, rootCargoToml, rootCargoLock, buildScript]);

      const isRustInput = (file: string) => {
        const f = normalizePath(path.resolve(file));
        return (
          f === rootCargoToml ||
          f === rootCargoLock ||
          f === buildScript ||
          f.startsWith(rustSrcDir + '/')
        );
      };

      const toRelativePath = (file: string): string => normalizePath(path.relative(repoRoot, file));

      const info = (message: string) => {
        server.config.logger.info(`\x1b[36m[rust-wasm]\x1b[0m ${message}`);
      };

      const error = (message: string) => {
        server.config.logger.error(`\x1b[31m[rust-wasm]\x1b[0m ${message}`);
      };

      const logStream = (prefix: string, chunk: Buffer) => {
        const lines = chunk
          .toString()
          .split(/\r?\n/)
          .map((line) => line.trimEnd())
          .filter((line) => line.length > 0);

        for (const line of lines) {
          info(`${prefix} ${line}`);
        }
      };

      let building = false;
      let queued = false;
      let queuedReason: string | null = null;

      const runBuild = (reason: string) => {
        if (building) {
          queued = true;
          queuedReason = reason;
          info(`⏳ build running, queued next rebuild (${reason})`);
          return;
        }
        building = true;

        const startedAt = Date.now();
        info(`🚧 rebuilding wasm (${reason})`);

        const child = spawn('bash', ['../scripts/build_web_wasm.sh'], {
          cwd: webRoot,
          stdio: 'pipe',
        });

        let stderrBuffer = '';
        child.stderr.on('data', (c: Buffer) => {
          stderrBuffer += c.toString();
          logStream('stderr:', c);
        });

        child.stdout.on('data', (c: Buffer) => {
          logStream('stdout:', c);
        });

        child.on('close', (code) => {
          building = false;

          if (code === 0) {
            const elapsed = ((Date.now() - startedAt) / 1000).toFixed(2);
            info(`✅ rebuild completed in ${elapsed}s; refreshing page`);
            server.ws.send({ type: 'full-reload', path: '*' });
          } else {
            const details = stderrBuffer.trim();
            error(
              details
                ? `❌ rebuild failed (exit ${code ?? 'unknown'})\n${details}`
                : `❌ rebuild failed with exit code ${code ?? 'unknown'}`
            );
          }

          if (queued) {
            queued = false;
            const nextReason = queuedReason ?? 'queued-change';
            queuedReason = null;
            runBuild(nextReason);
          }
        });
      };

      runBuild('initial-start');

      server.watcher.on('all', (event, file) => {
        if (!['add', 'change', 'unlink'].includes(event)) return;
        if (!isRustInput(file)) return;
        const normalized = normalizePath(path.resolve(file));
        const relative = toRelativePath(normalized);
        runBuild(`${event}:${relative}`);
      });
    },
  };
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, '.', '');

  return {
    plugins: [svelte(), rustWasmDevPlugin()],
    base: normalizeBasePath(env.VITE_BASE_PATH),
    define: {
      global: 'globalThis'
    },
    optimizeDeps: {
      esbuildOptions: {
        define: {
          global: 'globalThis'
        }
      }
    },
    server: {
      host: '0.0.0.0',
      port: 5173,
    }
  };
});
