import type { WebsocketHandler } from '@fastify/websocket';
import type { FastifyPluginAsync, RouteShorthandOptions } from 'fastify';
import * as ws from 'ws';

interface CreateWebSocketHandlerConfig {
  onConnection: (ws: ws.WebSocket) => void;
  onDisconnect: () => void;
}

export interface WebSocketDelegate {
  send: (code: string) => void;
}

export function createWebSocketHandler(config: CreateWebSocketHandlerConfig): {
  plugin;
  delegate: WebSocketDelegate;
} {
  let _socket: ws.WebSocket | null = null;

  const plugin: FastifyPluginAsync = async function (fastifyInstance) {
    const handler: WebsocketHandler = (socket) => {
      _socket = socket;

      socket.on('close', () => {
        _socket = null;
        config.onDisconnect();
      });

      config.onConnection(socket);
    };

    fastifyInstance.get(
      '/hot',
      // FIXME: Type of `websocket` is not correct.
      { websocket: true } as RouteShorthandOptions,
      handler,
    );
  };

  const delegate: WebSocketDelegate = {
    send(code: string) {
      if (_socket == null) {
        return;
      }

      _socket.send(code);
    },
  };

  return { plugin, delegate };
}
