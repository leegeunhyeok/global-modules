import * as esbuild from 'esbuild';
import assert from 'node:assert';
import path from 'node:path';
import type pino from 'pino';

let logger: pino.BaseLogger | null = null;
let cachedBuildResult: esbuild.BuildResult | null = null;
const CLIENT_SOURCE = path.resolve(__dirname, '../client/index.ts');

function setLogger<T extends pino.BaseLogger>(_logger: T) {
  logger = _logger;
}

function invalidateCache() {
  cachedBuildResult = null;
}

async function getBundle() {
  if (cachedBuildResult) {
    logger?.info('Bundler :: cache hit');
  } else {
    logger?.info('Bundler :: build triggered');
    cachedBuildResult = await build();
  }

  return getSource(cachedBuildResult);
}

async function build() {
  const buildResult = await esbuild.build({
    entryPoints: [CLIENT_SOURCE],
    bundle: true,
    sourcemap: true,
    metafile: false,
    write: false,
    inject: [], // TODO
    plugins: [], // TODO
  });

  await printMessages(buildResult);

  return buildResult;
}

async function printMessages(buildResult: esbuild.BuildResult) {
  const messages = (
    await Promise.all([
      esbuild.formatMessages(buildResult.errors, { kind: 'error' }),
      esbuild.formatMessages(buildResult.warnings, { kind: 'warning' }),
    ])
  ).flat();

  messages.forEach(console.log);
}

function getSource(buildResult: esbuild.BuildResult) {
  const data = buildResult.outputFiles?.[0].contents;

  assert(data, 'invalid bundle result');

  return data;
}

export const Bundler = { setLogger, invalidateCache, getBundle };
