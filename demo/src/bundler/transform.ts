import globalModulePlugin, {
  Phase,
  PluginConfig,
} from '@global-modules/swc-plugin';
import * as swc from '@swc/core';

export async function transform(
  source: string,
  filename: string,
  globalModuleConfig: PluginConfig,
) {
  const { code } = await swc.transform(source, {
    filename,
    configFile: false,
    jsc: {
      target: 'es5',
      parser: {
        syntax: 'typescript',
        tsx: true,
      },
      transform: {
        react: {
          runtime: 'classic',
          development: true,
          // @ts-ignore -- wrong type definition
          refresh: {
            refreshReg: 'window.__hot.reactRefresh.register',
            refreshSig: 'window.__hot.reactRefresh.getSignature',
          } as unknown,
        },
      },
      // External helpers are not allowed in the runtime phase
      // because helpers will be injected with import statements by the swc.
      externalHelpers: globalModuleConfig.phase === Phase.Bundle,
      experimental: {
        plugins: [[globalModulePlugin, globalModuleConfig]],
      },
    },
  });

  return code;
}
