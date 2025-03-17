import globalModulePlugin, { PluginConfig } from '@global-modules/swc-plugin';
import * as swc from '@swc/core';

const esModuleLexer = require('es-module-lexer');

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
          runtime: 'automatic',
          development: true,
          // @ts-ignore -- wrong type definition
          refresh: {
            refreshReg: 'window.$$reactRefresh$$.register',
            refreshSig: 'window.$$reactRefresh$$.getSignature',
          },
        },
      },
      // External helpers are not allowed in the runtime phase
      // because '@swc/helpers' will be injected by the swc with import statements after the plugin transformation.
      externalHelpers: globalModuleConfig.runtime === false,
      experimental: {
        plugins: [[globalModulePlugin, globalModuleConfig]],
      },
    },
  });

  return code;
}

/**
 * when `jsc.transform.react.runtime` is set to `automatic`,
 * the jsx runtime code will be injected with `import` statements after the its plugins.
 * It means that the transform result is cannot be evaluated directly.
 *
 * This function transforms the jsx runtime code to use the global context instead of `import` statements.
 */
export async function transformJsxRuntime(
  source: string,
  jsxRuntimeId: string,
) {
  const [imports] = esModuleLexer.parse(source);

  if (imports.length !== 1) {
    throw new Error('invalid input source');
  }

  const { n, ss, se } = imports[0];

  if (n !== 'react/jsx-dev-runtime') {
    throw new Error(`invalid module source: ${n}`);
  }

  // eg. `import { jsxDEV as _jsxDEV } from "react/jsx-dev-runtime"`
  const statement = source.slice(ss, se);
  const members: string[] = [];

  const ast = await swc.parse(statement);
  const node = ast.body[0];

  if (node.type === 'ImportDeclaration') {
    node.specifiers.forEach((specifier) => {
      if (specifier.type === 'ImportSpecifier') {
        members.push(
          specifier.imported
            ? `${specifier.imported.value}: ${specifier.local.value}`
            : specifier.local.value,
        );
      }
    });
  }
  return [
    // `$$jsxDevRuntime$$`: See `runtime/index.js`
    `var { ${members.join(', ')} } = global.__modules.require(${JSON.stringify(jsxRuntimeId)});`,
    source.replace(statement, ''),
  ].join('\n');
}
