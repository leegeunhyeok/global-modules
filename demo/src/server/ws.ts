import type { WebsocketHandler } from '@fastify/websocket';
import * as ws from 'ws';

interface CreateWebSocketHandlerConfig {
  onConnection: (ws: WebSocket) => void;
  onDisconnect: () => void;
}

export interface WebSocketDelegate {
  send: (code: string) => void;
}

export function createWebSocketHandler(config: CreateWebSocketHandlerConfig): {
  plugin: WebsocketHandler;
  delegate: WebSocketDelegate;
} {
  let socket: ws.WebSocket | null = null;

  const plugin = async function (fastifyInstance) {
    fastifyInstance.get('/hot', { websocket: true }, (connection) => {
      socket = connection.socket;
      config.onConnection(connection.socket);

      connection.socket.on('close', () => {
        config.onDisconnect();
      });
    });
  };

  const delegate: WebSocketDelegate = {
    send(code: string) {
      if (socket == null) {
        return;
      }

      socket.send(code);
    },
  };

  return { plugin, delegate };
}
