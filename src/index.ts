import { env } from 'node:process';
import path from 'node:path';
import type { BuildOptions, BuildResultV3 } from '@vercel/build-utils';
import {
  FileFsRef,
  debug,
  download,
  glob,
  createLambda,
} from '@vercel/build-utils';
import execa from 'execa';
import { installRustToolchain } from './lib/rust-toolchain';
import type { Runtime } from './lib/runtime';
import {
  getCargoMetadata,
  findBinaryName,
  findCargoWorkspace,
  findCargoBuildConfiguration,
} from './lib/cargo';
import {
  assertEnv,
  getExecutableName,
  gatherExtraFiles,
  runUserScripts,
} from './lib/utils';

type RustEnv = Record<'RUSTFLAGS' | 'PATH', string>;

async function buildHandler(options: BuildOptions): Promise<BuildResultV3> {
  const BUILDER_DEBUG = Boolean(process.env.VERCEL_BUILDER_DEBUG ?? false);
  const { files, entrypoint, workPath, config, meta } = options;

  await installRustToolchain();

  debug('Creating file system');
  const downloadedFiles = await download(files, workPath, meta);
  const entryPath = downloadedFiles[entrypoint].fsPath;

  const HOME = assertEnv(env.HOME);
  const PATH = assertEnv('PATH');

  const rustEnv: RustEnv = {
  PATH: `${path.join(HOME, '.cargo/bin')}:${env.PATH}`,
  RUSTFLAGS: [env.RUSTFLAGS].filter(Boolean).join(' '),
  };


  const cargoWorkspace = await findCargoWorkspace({
    env: rustEnv,
    cwd: path.dirname(entryPath),
  });

  const binaryName = findBinaryName(cargoWorkspace, entryPath);
  const cargoBuildConfiguration = await findCargoBuildConfiguration(
    cargoWorkspace,
  );
  const buildTarget = cargoBuildConfiguration?.build.target ?? '';

  await runUserScripts(workPath);

  const extraFiles = await gatherExtraFiles(config.includeFiles, workPath);

  debug(`Running \`cargo build\` for \`${binaryName}\``);
  try {
    await execa(
      'cargo',
      ['build', '--bin', binaryName].concat(
        BUILDER_DEBUG ? ['--verbose'] : ['--quiet', '--release'],
      ),
      {
        cwd: workPath,
        env: rustEnv,
        stdio: 'inherit',
      },
    );
  } catch (err) {
    debug(`Running \`cargo build\` for \`${binaryName}\` failed`);
    throw err;
  }

  debug(`Building \`${binaryName}\` for \`${process.platform}\` completed`);

  let { target_directory: targetDirectory } = await getCargoMetadata({
    cwd: process.cwd(),
    env: rustEnv,
  });

  targetDirectory = path.join(targetDirectory, buildTarget);

  const bin = path.join(
    targetDirectory,
    BUILDER_DEBUG ? 'debug' : 'release',
    getExecutableName(binaryName),
  );

  const bootstrap = getExecutableName('bootstrap');
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
