import { existsSync } from 'node:fs';
import { join } from 'node:path';

const wasmPath = join(
  __dirname,
  './target/wasm32-wasi/release/swc_plugin_global_modules.wasm',
);

if (existsSync(wasmPath)) {
  throw new Error('wasm binary not found');
}

export default wasmPath;
