import path from 'node:path';

export const CLIENT_SOURCE_BASE = path.resolve(__dirname, '../client');
export const CLIENT_SOURCE_ENTRY = path.join(
  CLIENT_SOURCE_BASE,
  'src/index.js',
);

export const BUNDLE_FILE_PATH = '/bundle.js';
