import * as fs from 'node:fs';
import type { DependencyGraph } from 'esbuild-dependency-graph';

export function createGraphLoaderPlugin({ graph }: { graph: DependencyGraph }) {
  return {
    name: 'dependency-graph-loader',
    setup: ({ onEnd }) => {
      onEnd((result) => {
        if (result.metafile == null) {
          throw new Error('Metafile is not found');
        }

        graph.load(result.metafile);

        fs.writeFileSync(
          './graph.json',
          JSON.stringify(graph, null, 2),
          'utf-8',
        );
      });
    },
  };
}
