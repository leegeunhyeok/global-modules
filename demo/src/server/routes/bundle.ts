import { RouteHandler } from './types';
import { BUNDLE_FILE_PATH } from '../../shared';
import { Bundler } from '../../bundler';

export const bundleRoute: RouteHandler = [
  BUNDLE_FILE_PATH,
  async (_request, reply) => {
    const bundle = await Bundler.getInstance().getBundle();

    reply.type('application/javascript').send(bundle);
  },
];
