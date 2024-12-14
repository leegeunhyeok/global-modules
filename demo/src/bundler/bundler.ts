import assert from 'node:assert';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as esbuild from 'esbuild';
import type pino from 'pino';
import type { DependencyManager } from '@global-modules/esbuild-plugin';
import { createTransformPlugin } from './transformPlugin';

const CLIENT_SOURCE = path.resolve(__dirname, '../client/index.ts');

class Bundler {
  private logger: pino.BaseLogger | null = null;
  private cachedBuildResult: esbuild.BuildResult | null = null;
  private dependencyManager: DependencyManager | null = null;

  private async build() {
    const transformPlugin = createTransformPlugin();

    this.dependencyManager = transformPlugin.dependencyManager;

    const buildResult = await esbuild.build({
      entryPoints: [CLIENT_SOURCE],
      bundle: true,
      sourcemap: true,
      metafile: false,
      write: false,
      banner: {
        // Inject `@global-modules/runtime` as a prelude script.
        js: await this.getPreludeScript(),
      },
      plugins: [transformPlugin.plugin],
    });

    return buildResult;
  }

  private async getPreludeScript() {
    const source = await fs.promises.readFile(
      require.resolve('@global-modules/runtime'),
      {
        encoding: 'utf-8',
      },
    );

    return source;
  }

  private getSource(buildResult: esbuild.BuildResult) {
    const data = buildResult.outputFiles?.[0].contents;

    assert(data, 'invalid bundle result');

    return data;
  }

  async getBundle() {
    if (this.cachedBuildResult) {
      this.logger?.info('Bundler :: cache hit');
    } else {
      this.logger?.info('Bundler :: build triggered');
      this.cachedBuildResult = await this.build();
    }

    const bundleResult = this.cachedBuildResult;

    return this.getSource(bundleResult);
  }

  setLogger<T extends pino.BaseLogger>(logger: T) {
    this.logger = logger;
  }

  invalidateCache() {
    this.cachedBuildResult = null;
  }
}

export const bundler = new Bundler();
