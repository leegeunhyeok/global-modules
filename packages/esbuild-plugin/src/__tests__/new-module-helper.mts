/* eslint-disable quotes -- allow template literal */
import * as fs from 'node:fs/promises';
import * as path from 'node:path';

const ROOT = import.meta.dirname;
const FIXTURE_ROOT = path.resolve(ROOT, '__fixtures__');
const NEW_MODULE_PATH = path.join(FIXTURE_ROOT, 'new-module.ts');

const INITIAL_CODE = [
  `import { deps1 } from './new-deps-1';`,
  `import { deps2 } from './new-deps-2';`,
  'export default [deps1, deps2];',
].join('\n');

const UPDATED_CODE = [
  `import { deps1 } from './new-deps-1';`,
  `import { deps2 } from './new-deps-2';`,
  `import { deps3 } from './new-deps-3';`,
  'export default [deps1, deps2, deps3];',
].join('\n');

export async function generateInitialModule(): Promise<void> {
  await fs.writeFile(NEW_MODULE_PATH, INITIAL_CODE, 'utf-8');
}

export async function generateUpdatedModule(): Promise<void> {
  await fs.writeFile(NEW_MODULE_PATH, UPDATED_CODE, 'utf-8');
}
