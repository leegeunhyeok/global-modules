import globalModulePlugin, {
  Phase,
  PluginConfig,
} from '@global-modules/swc-plugin';
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

/**
 * when `jsc.transform.react.runtime` is set to `automatic`,
 * the jsx runtime code will be injected with `import` statements after the its plugins.
 * It means that the transform result is cannot be evaluated directly.
 *
 * This function transforms the jsx runtime code to use the global context instead of `import` statements.
 */
export async function transformJsxRuntime(source: string) {
  const [imports] = esModuleLexer.parse(source);

  if (imports.length !== 1) {
    throw new Error('invalid input source');
  }

  const { n, ss, se } = imports[0];

  if (n !== 'react/jsx-dev-runtime') {
    throw new Error(`invalid module source: ${n}`);
  }

  // `import { jsxDEV as _jsxDEV } from "react/jsx-dev-runtime"`
  const statement = source.slice(ss, se);
  const members: string[] = [];

  if (statement.includes('jsxDEV')) {
    members.push('jsxDev: _jsxDev');
  }

  if (statement.includes('Fragment')) {
    members.push('Fragment: _Fragment');
  }

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

  // See `runtime/index.js`
  return source.replace(
    statement,
    `var { ${members.join(', ')} } = window.__hot.jsxDevRuntime;`,
  );
}
