import assert from 'node:assert';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { Event } from '@parcel/watcher';
import * as esbuild from 'esbuild';
import * as esresolve from 'esresolve';
import { DependencyGraph, type Module } from 'esbuild-dependency-graph';
import type pino from 'pino';
import * as watcher from './watcher';
import { createTransformPlugin } from './transformPlugin';
import { WebSocketDelegate } from '../server/ws';
import { transform, transformJsxRuntime } from './transform';
import { CLIENT_SOURCE_BASE, CLIENT_SOURCE_ENTRY } from '../shared';
import { metafilePlugin } from './metafilePlugin';
import { loadSource } from '../utils/loadSource';
import { wrapWithIIFE } from '../utils/wrapWithIIFE';
import { applyModule } from './templates';
import { createGraphLoaderPlugin } from './graphLoaderPlugin';
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
  private graph: DependencyGraph;
  private jsxDevRuntimeModuleId: number | null = null;

  public static getInstance() {
    if (Bundler.instance === null) {
      Bundler.instance = new Bundler();
    }
    return Bundler.instance;
  }

  private constructor() {
    this.graph = new DependencyGraph({});
  }

  private async build() {
    const buildResult = await esbuild.build({
      entryPoints: [CLIENT_SOURCE_ENTRY],
      bundle: true,
      sourcemap: true,
      metafile: true,
      write: false,
      inject: [path.join(__dirname, 'runtime/index.js')],
      banner: {
        js: await this.getPreludeScript(),
      },
      plugins: [
        createGraphLoaderPlugin({ graph: this.graph }),
        createTransformPlugin({
          resolveId: (id) => {
            const module = this.graph.hasModule(id)
              ? this.graph.getModule(id)
              : this.graph.addModule(id, { dependencies: [] });

            if (id.endsWith('react/jsx-dev-runtime.js')) {
              this.jsxDevRuntimeModuleId = module.id;
            }

            return module.id;
          },
        }),
        metafilePlugin,
      ],
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
      if (this.graph.size === 0) {
        // To avoid unnecessary transform when the graph is not loaded.
        return;
      }

      let affectedModule: Module | null = null;

      switch (event.type) {
        case 'create':
        case 'update': {
          const resolveResults = await esresolve.resolveFrom(event.path);
          const dependencies = resolveResults.map((result) => ({
            key: result.path,
            source: result.request,
          }));

          const graphMethod =
            event.type === 'create'
              ? this.graph.addModule
              : this.graph.updateModule;

          affectedModule = graphMethod.bind(this.graph)(event.path, {
            dependencies,
          });

          break;
        }

        case 'delete':
          this.graph.removeModule(event.path);
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
    const jsxDevRuntimeModuleId = this.jsxDevRuntimeModuleId;

    if (this.delegate == null) {
      console.warn('Websocket delegate is not initialized');
      return;
    }

    if (jsxDevRuntimeModuleId == null) {
      console.warn('JSX dev runtime module id is not set');
      return;
    }

    const inverseDependencies = this.graph.inverseDependenciesOf(baseModule.id);

    const t0 = performance.now();

    const moduleId = baseModule.id.toString();
    const originCode = await fs.promises.readFile(
      path.resolve(CLIENT_SOURCE_BASE, baseModule.path),
      {
        encoding: 'utf-8',
      },
    );
    const transformedCode = await transform(
      originCode,
      path.basename(module.path),
      {
        id: moduleId,
        runtime: true,
        paths: Object.fromEntries(
          baseModule.dependencies.map((dependency) => [
            dependency.source,
            dependency.id.toString(),
          ]),
        ),
      },
    )
      .then((code) =>
        transformJsxRuntime(code, jsxDevRuntimeModuleId.toString()),
      )
      .then(wrapWithIIFE);

    const t1 = performance.now();

    const code = [
      transformedCode,
      ...inverseDependencies.map((module) =>
        applyModule(
          module.id.toString(),
          Object.fromEntries(
            module.dependencies.map((dependency) => [
              dependency.source,
              dependency.id.toString(),
            ]),
          ),
        ),
      ),
    ].join('\n');

    fs.writeFileSync(
      './graph.json',
      JSON.stringify(this.graph, null, 2),
      'utf-8',
    );

    console.log(`[HMR] Module transformed in ${Math.floor(t1 - t0)}ms`);

    this.delegate.send(
      JSON.stringify({
        type: 'update',
        id: baseModule.id,
        body: code,
      }),
    );
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
