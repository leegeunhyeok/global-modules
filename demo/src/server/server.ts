import fastify from 'fastify';
import fastifyWebsocket from '@fastify/websocket';
import { handler as webSocketHandler } from './ws';
import { indexRoute, bundleRoute } from './routes';

const server = fastify({
  logger: {
    transport: {
      target: 'pino-pretty',
      options: {
        translateTime: 'yyyy-mm-dd HH:MM:ss Z',
        ignore: 'pid,hostname',
      },
    },
  },
});

server
  .get(...indexRoute)
  .get(...bundleRoute)
  .register(fastifyWebsocket)
  .register(webSocketHandler);

export { server };
