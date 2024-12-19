import fastify, { FastifyInstance } from 'fastify';
import fastifyWebsocket from '@fastify/websocket';
import { createWebSocketHandler, WebSocketDelegate } from './ws';
import { indexRoute, bundleRoute } from './routes';

export function createServer(): FastifyInstance & {
  delegate: WebSocketDelegate;
} {
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

  const wsHandler = createWebSocketHandler({
    onConnection: (socket) => {
      server.log.info(socket, 'WebSocket :: Connected');
    },
    onDisconnect: () => {
      server.log.info('WebSocket :: Disconnected');
    },
  });

  server
    .get(...indexRoute)
    .get(...bundleRoute)
    .register(fastifyWebsocket)
    .register(wsHandler.plugin);

  return Object.assign(server, { delegate: wsHandler.delegate });
}
