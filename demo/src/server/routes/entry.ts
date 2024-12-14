import * as fs from 'node:fs';
import * as path from 'node:path';
import { RouteHandler } from './types';
import { BUNDLE_FILE_PATH } from '../../shared';

export const indexRoute: RouteHandler = [
  '/',
  async (_request, reply) => {
    const content = await fs.promises.readFile(
      path.resolve(__dirname, '../index.html'),
      {
        encoding: 'utf-8',
      },
    );

    reply.type('text/html').send(replaceBundlePath(content));
  },
];

function replaceBundlePath(html: string) {
  return html.replace(/__BUNDLE_FILE_PATH__/g, BUNDLE_FILE_PATH);
}