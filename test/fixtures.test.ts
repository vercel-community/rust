import { join } from 'node:path';
import ms from 'ms';
import execa from 'execa';
import { writeJSON } from 'fs-extra';
import fetch from 'node-fetch';

const vercelFileName = 'test.vercel.json';
const pkgRootFile = `file:${process.cwd()}`;
const readyRegex = /Ready!\s+Available at (?<url>https?:\/\/\w+:\d+)/;

jest.setTimeout(ms('50m'));

interface Config {
  builds: {
    use: string;
  }[];
}

interface Probe {
  status: number;
  path: string;
  mustContain: string;
}

interface ProbesConf {
  probes: Probe[];
}

async function importJSON<T>(path: string): Promise<T> {
  return (await import(path)) as unknown as Promise<T>;
}

async function injectConf(confPath: string): Promise<Config> {
  const conf = await importJSON<Config>(join(confPath, vercelFileName));

  conf.builds[0].use = pkgRootFile;

  await writeJSON(join(confPath, 'vercel.json'), conf, {
    spaces: 2,
    EOL: '\n',
  });

  return conf;
}

async function checkProbes(baseUrl: string, probes: Probe[]): Promise<void> {
  for (const probe of probes) {
    // eslint-disable-next-line no-await-in-loop
    const response = (await fetch(`${baseUrl}${probe.path}`)) as unknown as {
      status: number;
      text: () => Promise<string>;
    };

    const status = response.status;
    // eslint-disable-next-line no-await-in-loop
    const text = await response.text();

    if (probe.status) {
      expect(status).toBe(probe.status);
    }

    if (probe.mustContain) {
      expect(text).toContain(probe.mustContain);
    }
  }
}

function getVercelProcess(dir: string): execa.ExecaChildProcess {
  const defaultArgs = ['dev', '--yes'];

  if (process.env.VERCEL_TOKEN) {
    defaultArgs.push('--token', process.env.VERCEL_TOKEN);
  }

  return execa('vercel', [...defaultArgs, dir]);
}

function isReady(vercelServ: execa.ExecaChildProcess): Promise<string> {
  return new Promise((resolve) => {
    vercelServ.stderr?.on('data', (d: Buffer) => {
      const res = readyRegex.exec(d.toString());
      if (res?.groups?.url) {
        resolve(res.groups.url);
      }
    });
    vercelServ.stderr?.pipe(process.stderr);
  });
}

async function testFixture(fixture: string): Promise<'ok'> {
  /* await injectConf(join(__dirname, 'fixtures', fixture)); */

  const { probes } = await importJSON<ProbesConf>(
    join(__dirname, 'fixtures', fixture, 'probes.json'),
  );

  const vercelProcess = getVercelProcess(join(__dirname, 'fixtures', fixture));
  const baseUrl = await isReady(vercelProcess);

  await checkProbes(baseUrl, probes);
  vercelProcess.cancel();
  vercelProcess.stdout?.destroy();
  vercelProcess.stderr?.destroy();

  return Promise.resolve('ok');
}

describe('vercel-rust', () => {
  it('deploy 01-include-files', async () => {
    await expect(testFixture('01-include-files')).resolves.toBe('ok');
  });
  it('deploy 02-with-utility', async () => {
    await expect(testFixture('02-with-utility')).resolves.toBe('ok');
  });
  it('deploy 03-with-function', async () => {
    await expect(testFixture('03-with-function')).resolves.toBe('ok');
  });
  it('deploy 04-with-parameter', async () => {
    await expect(testFixture('04-with-parameter')).resolves.toBe('ok');
  });
  it('deploy 05-with-similar-entrypaths', async () => {
    await expect(testFixture('05-with-similar-entrypaths')).resolves.toBe('ok');
  });
});
