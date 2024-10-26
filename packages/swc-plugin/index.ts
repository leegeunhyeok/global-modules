import { existsSync } from 'node:fs';
import { join } from 'node:path';

const wasmPath = join(
  __dirname,
  './target/wasm32-wasi/release/swc_plugin_global_module.wasm',
);

if (existsSync(wasmPath)) {
  throw new Error('wasm binary not found');
}

// eslint-disable-next-line import/no-default-export -- ignore
export default wasmPath;
