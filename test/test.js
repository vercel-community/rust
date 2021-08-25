/* global beforeAll, expect, it, jest */
const ms = require('ms');
const path = require('path');
const execa = require('execa');
const fs = require('fs-extra');
const fetch = require('node-fetch');

const vercelFileName = 'test.vercel.json';
const pkgRootFile = 'file:' + process.cwd();
const readyRegex = /Ready\!\s+Available at (https?:\/\/\w+:\d+)/;

jest.setTimeout(ms('50m'));

async function injectConf(confPath) {
  const conf = require(path.join(confPath, vercelFileName));
  conf.builds[0].use = pkgRootFile;

  await fs.writeJSON(path.join(confPath, 'vercel.json'), conf, {
    spaces: 2,
    EOL: '\n'
  });

  return conf;
}

async function checkProbes(baseUrl, probes) {
  for (const probe of probes) {
    const response = await fetch(`${baseUrl}${probe.path}`);
    const status = response.status;
    const text = await response.text();

    if (probe.status) {
      expect(status).toBe(probe.status);
    }

    if (probe.mustContain) {
      expect(text).toContain(probe.mustContain);
    }
  }
}

function getVercelProcess(dir) {
  const defaultArgs = ['dev', '--confirm'];

  if (process.env.VERCEL_TOKEN) {
    defaultArgs.push('--token', process.env.VERCEL_TOKEN);
  }

  return execa('vercel', [...defaultArgs, dir]);
}

function isReady(vercelServ) {
  return new Promise((resolve) => {
    vercelServ.stderr.on('data', (d) => {
      const res = readyRegex.exec(d.toString());

      if (res && res[1]) {
        resolve(res[1]);
      }
    });

    vercelServ.stderr.pipe(process.stderr);
  });
}

async function testFixture(fixture) {
  const vercelConf = await injectConf(
    path.join(__dirname, 'fixtures', fixture)
  );
  const vercelProcess = getVercelProcess(
    path.join(__dirname, 'fixtures', fixture)
  );
  const baseUrl = await isReady(vercelProcess);
  await checkProbes(baseUrl, vercelConf.probes);
  vercelProcess.cancel();
}

describe('vercel-rust', () => {
  it('Deploy 01-include-files', async () => {
    await testFixture('01-include-files');
  });

  it('Deploy 02-with-utility', async () => {
    await testFixture('02-with-utility');
  });

  it('Deploy 03-with-function', async () => {
    await testFixture('03-with-function');
  });

  it('Deploy 04-with-parameter', async () => {
    await testFixture('04-with-parameter');
  });

  it('Deploy 05-preconfigured-binary', async () => {
    await testFixture('05-preconfigured-binary');
  });
});
