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
import { transform, transformJsxRuntime } from './transform';
import { Phase } from '@global-modules/swc-plugin';
import { CLIENT_SOURCE_BASE, CLIENT_SOURCE_ENTRY } from '../shared';
import { metafilePlugin } from './metafilePlugin';
import { loadSource } from '../utils/loadSource';
import { wrapWithIIFE } from '../utils/wrapWithIIFE';
const esModuleLexer = require('es-module-lexer');

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
      write: false,
      inject: [path.join(__dirname, 'runtime/index.js')],
      banner: {
        js: await this.getPreludeScript(),
      },
      plugins: [transformPlugin.plugin, metafilePlugin],
    });

    return buildResult;
  }

  private async getPreludeScript() {
    const preludeSourcePaths = [
      require.resolve('@global-modules/runtime'),
      path.join(__dirname, 'runtime/hot-context.js'),
    ];

    return Promise.all(preludeSourcePaths.map(loadSource)).then((sources) =>
      sources.map(wrapWithIIFE).join('\n'),
    );
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
        await this.transformAffectedModules(affectedModule).catch((error) => {
          this.logger?.error(error?.message ?? 'unknown transform error');
        });
      }
    });
  }

  private async transformAffectedModules(baseModule: Module) {
    if (this.dependencyManager == null || this.delegate == null) {
      return;
    }

    let invalid = false;

    const inverseDependencies = this.dependencyManager.inverseDependenciesOf(
      baseModule.id,
    );

    const t0 = performance.now();
    const transformedCodeList = await Promise.all(
      [baseModule, ...inverseDependencies].map(async (module) => {
        const code = await fs.promises.readFile(module.path, {
          encoding: 'utf-8',
        });

        const imports = Object.entries(module.meta.imports).reduce(
          (prev, [original, value]) => {
            return { ...prev, [original]: value.id };
          },
          {},
        );

        const transformedCode = await transform(
          code,
          path.basename(module.path),
          {
            id: module.id,
            phase: Phase.Runtime,
            paths: imports,
          },
        ).then(transformJsxRuntime);

        return wrapWithIIFE(transformedCode);
      }),
    );

    if (invalid) {
      console.log('[HMR] Invalid modules (will be reloaded)');

      this.delegate.send(JSON.stringify({ type: 'reload' }));
    } else {
      const t1 = performance.now();
      const code = transformedCodeList.join('\n\n');
      console.log(
        `[HMR] ${transformedCodeList.length} module(s) transformed in ${Math.floor(t1 - t0)}ms`,
      );

      this.delegate.send(
        JSON.stringify({
          type: 'update',
          id: baseModule.id,
          body: code,
        }),
      );
    }
  }

  async initialize(config: BundlerConfig) {
    esModuleLexer.init;
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
