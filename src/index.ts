import fs from 'node:fs';
import path from 'node:path';
import type { BuildOptions, BuildResultV3 } from '@vercel/build-utils';
import {
  FileFsRef,
  debug,
  download,
  glob,
  runShellScript,
  createLambda,
} from '@vercel/build-utils';
import execa from 'execa';
import { installRustToolchain } from './lib/rust-toolchain';
import type { Runtime } from './lib/runtime';

type RustEnv = Record<'RUSTFLAGS' | 'PATH', string>;

function assertEnv(name: string): string {
  if (!process.env[name]) {
    throw new Error(`Missing ENV variable process.env.${name}`);
  }

  return process.env[name] as unknown as string;
}

async function runUserScripts(dir: string): Promise<void> {
  const buildScriptPath = path.join(dir, 'build.sh');
  const buildScriptExists = fs.existsSync(buildScriptPath);

  if (buildScriptExists) {
    debug('Running `build.sh`');
    await runShellScript(buildScriptPath);
  }
}

async function gatherExtraFiles(
  globMatcher: string | string[] | undefined,
  entrypoint: string,
): Promise<Record<string, FileFsRef>> {
  if (!globMatcher) return {};

  debug('Gathering extra files for the fs');

  const entryDir = path.dirname(entrypoint);

  if (Array.isArray(globMatcher)) {
    const allMatches = await Promise.all(
      globMatcher.map((pattern) => glob(pattern, entryDir)),
    );

    return allMatches.reduce((acc, matches) => ({ ...acc, ...matches }), {});
  }

  return glob(globMatcher, entryDir);
}

async function buildHandler(options: BuildOptions): Promise<BuildResultV3> {
  const BUILDER_DEBUG = Boolean(process.env.VERCEL_BUILDER_DEBUG ?? false);
  const { files, entrypoint, workPath, config, meta } = options;

  await installRustToolchain();

  debug('Creating file system');
  const downloadedFiles = await download(files, workPath, meta);
  const entryPath = downloadedFiles[entrypoint].fsPath;

  const HOME = assertEnv('HOME');
  const PATH = assertEnv('PATH');

  const rustEnv: RustEnv = {
    PATH: `${path.join(HOME, '.cargo/bin')}:${PATH}`,
    RUSTFLAGS: [process.env.RUSTFLAGS].filter(Boolean).join(' '),
  };

  // The binary name is the name of the entrypoint file
  // We assume each binary is specified correctly with `[[bin]]` in `Cargo.toml`
  const binaryName = path.basename(entryPath, '.rs');

  await runUserScripts(workPath);
  const extraFiles = await gatherExtraFiles(config.includeFiles, entryPath);

  debug(`Running \`cargo build\` for \`${binaryName}\``);
  try {
    await execa(
      'cargo',
      ['build', '--bin', binaryName].concat(
        BUILDER_DEBUG ? ['--verbose'] : ['--quiet', '--release'],
      ),
      {
        cwd: process.cwd(),
        env: rustEnv,
        stdio: 'inherit',
      },
    );
  } catch (err) {
    debug(`Running \`cargo build\` for \`${binaryName}\` failed`);
    throw err;
  }

  debug(`Building \`${binaryName}\` completed`);

  // The compiled binary in Windows has the `.exe` extension
  const binExtension = process.platform === 'win32' ? '.exe' : '';
  const bootstrap = `bootstrap${binExtension}`;

  const targetPath = `target/${BUILDER_DEBUG ? 'debug' : 'release'}`;
  const bin = path.join(process.cwd(), `${targetPath}/${binaryName}`);

  const lambda = await createLambda({
    files: {
      ...extraFiles,
      [bootstrap]: new FileFsRef({ mode: 0o755, fsPath: bin }),
    },
    handler: bootstrap,
    runtime: 'provided',
  });

  return {
    output: lambda,
  };
}

// Reference -  https://github.com/vercel/vercel/blob/main/DEVELOPING_A_RUNTIME.md#runtime-developer-reference
const runtime: Runtime = {
  version: 3,
  build: buildHandler,
  prepareCache: async ({ workPath }) => {
    debug(`Caching \`${workPath}\``);
    const cacheFiles = await glob('target/**', workPath);

    // Convert this into a reduce
    for (const f of Object.keys(cacheFiles)) {
      const accept =
        /(?:^|\/)target\/release\/\.fingerprint\//.test(f) ||
        /(?:^|\/)target\/release\/build\//.test(f) ||
        /(?:^|\/)target\/release\/deps\//.test(f) ||
        /(?:^|\/)target\/debug\/\.fingerprint\//.test(f) ||
        /(?:^|\/)target\/debug\/build\//.test(f) ||
        /(?:^|\/)target\/debug\/deps\//.test(f);
      if (!accept) {
        delete cacheFiles[f];
      }
    }

    return cacheFiles;
  },
  shouldServe: async (options): Promise<boolean> => {
    return Promise.resolve(options.requestPath === options.entrypoint);
  },
};

export const { version, build, prepareCache, shouldServe } = runtime;
