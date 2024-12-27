import globalModulePlugin, { PluginConfig } from '@global-modules/swc-plugin';
import * as swc from '@swc/core';

export async function transform(
  source: string,
  filename: string,
  pluginConfig: PluginConfig,
) {
  const { code } = await swc.transform(source, {
    filename,
    configFile: false,
    jsc: {
      parser: {
        syntax: 'typescript',
        decorators: false,
        tsx: false,
      },
      target: 'es5',
      experimental: {
        plugins: [[globalModulePlugin, pluginConfig]],
      },
    },
  });

  return code;
}
