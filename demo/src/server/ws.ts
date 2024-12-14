import type { WebsocketHandler } from '@fastify/websocket';

export const handler: WebsocketHandler = async function (fastifyInstance) {
  fastifyInstance.get('/websocket', { websocket: true }, (connection, req) => {
    connection.socket.on('message', (message: string) => {
      console.log('Received:', message);
      connection.socket.send(`Echo: ${message}`);
    });

    connection.socket.on('close', () => {
      console.log('WebSocket connection closed');
    });
  });
};
