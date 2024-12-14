import assert from 'node:assert';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as esbuild from 'esbuild';
import type pino from 'pino';
import type { DependencyManager } from '@global-modules/esbuild-plugin';
import * as watcher from './watcher';
import { createTransformPlugin } from './transformPlugin';
import { Event } from '@parcel/watcher';

const CLIENT_SOURCE_BASE = path.resolve(__dirname, '../client');
const CLIENT_SOURCE_ENTRY = path.join(CLIENT_SOURCE_BASE, 'index.js');

interface BundlerConfig {
  logger: pino.BaseLogger;
}

export class Bundler {
  public static instance: Bundler | null = null;
  private logger: pino.BaseLogger | null = null;
  private cachedBuildResult: esbuild.BuildResult | null = null;
  private dependencyManager: DependencyManager | null = null;

  public static getInstance() {
    if (Bundler.instance === null) {
      Bundler.instance = new Bundler();
    }
    return Bundler.instance;
  }

  private constructor() {}

  private async build() {
    const transformPlugin = createTransformPlugin();

    this.dependencyManager = transformPlugin.dependencyManager;

    const buildResult = await esbuild.build({
      entryPoints: [CLIENT_SOURCE_ENTRY],
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

  private watchHandler(events: Event[]) {
    events.forEach(async (event) => {
      if (this.dependencyManager == null) {
        return;
      }

      switch (event.type) {
        case 'create':
          this.dependencyManager.addModule(event.path);
          break;

        case 'delete':
          this.dependencyManager.removeModule(event.path);
          break;

        case 'update':
          await this.dependencyManager.syncModule(event.path);
          break;
      }

      try {
        const existingModule = this.dependencyManager.getModule(event.path);

        this.logger?.info(existingModule, `Target: ${event.path}`);
      } catch {
        this.logger?.warn(`The ${event.path} module may have been removed`);
      }
    });
  }

  async initialize(config: BundlerConfig) {
    await watcher.watch(CLIENT_SOURCE_BASE, this.watchHandler.bind(this));

    this.logger = config.logger;
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
