import assert from 'node:assert';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as esbuild from 'esbuild';
import type pino from 'pino';
import type { DependencyManager, Module } from '@global-modules/esbuild-plugin';
import * as watcher from './watcher';
import { createTransformPlugin } from './transformPlugin';
import { Event } from '@parcel/watcher';
import { WebSocketDelegate } from '../server/ws';
import { transform } from './transform';
import { Phase } from '@global-modules/swc-plugin';

const CLIENT_SOURCE_BASE = path.resolve(__dirname, '../client');
const CLIENT_SOURCE_ENTRY = path.join(CLIENT_SOURCE_BASE, 'index.js');

interface BundlerConfig {
  delegate: WebSocketDelegate;
  logger: pino.BaseLogger;
}

export class Bundler {
  public static instance: Bundler | null = null;
  private delegate: WebSocketDelegate | null = null;
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
      let affectedModule: Module | null = null;

      if (this.dependencyManager == null) {
        return;
      }

      switch (event.type) {
        case 'create':
        case 'update':
          affectedModule = await this.dependencyManager.syncModule(event.path);
          break;

        case 'delete':
          this.dependencyManager.removeModule(event.path);
          break;
      }

      if (affectedModule) {
        this.logger?.info(affectedModule, `Affected module`);
        this.sendHMR(affectedModule);
      }
    });
  }

  private async sendHMR(module: Module) {
    if (this.dependencyManager == null || this.delegate == null) {
      return;
    }

    let invalid = false;

    const inverseDependencies = this.dependencyManager.inverseDependenciesOf(
      module.id,
    );

    const t0 = performance.now();
    const transformedCodeList = await Promise.all(
      [module, ...inverseDependencies].map(async (module) => {
        const code = await fs.promises.readFile(module.path, {
          encoding: 'utf-8',
        });

        if (
          module.meta?.imports &&
          module.dependencies.length !==
            Object.keys(module.meta?.imports ?? {}).length
        ) {
          invalid = true;
          console.warn('dependency meta is mismatch');
          return '';
        }

        const imports = Object.entries(module.meta.imports).reduce(
          (prev, [original, value]) => {
            return { ...prev, [original]: value.id };
          },
          {},
        );

        return transform(code, path.basename(module.path), {
          id: module.id,
          phase: Phase.Runtime,
          dependencyIds: imports,
        });
      }),
    );

    if (invalid) {
      console.log('window.reload();');
    } else {
      const t1 = performance.now();
      const code = transformedCodeList.join('\n\n');

      console.log(code);
      console.log(`[HMR] Module transformed in ${Math.floor(t1 - t0)}ms`);
    }
  }

  async initialize(config: BundlerConfig) {
    await watcher.watch(CLIENT_SOURCE_BASE, this.watchHandler.bind(this));

    this.logger = config.logger;
    this.delegate = config.delegate;
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
