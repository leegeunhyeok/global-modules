import assert from 'node:assert';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

const wasmPath = join(
  __dirname,
  '../target/wasm32-wasip1/release/swc_plugin_global_modules.wasm',
);

assert(existsSync(wasmPath), `wasm binary not found: ${wasmPath}`);

export default wasmPath;
export * from './types';
