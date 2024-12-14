import { RouteHandler } from './types';
import { Bundler } from '../../bundler';
import { BUNDLE_FILE_PATH } from '../../shared';

export const bundleRoute: RouteHandler = [
  BUNDLE_FILE_PATH,
  async (_request, reply) => {
    const bundle = await Bundler.getBundle();

    reply.type('application/javascript').send(bundle);
  },
];
