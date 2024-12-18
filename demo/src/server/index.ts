import { Bundler } from '../bundler';
import { createServer } from './server';

const PORT = 3000;

async function main() {
  const server = createServer();

  await Bundler.getInstance().initialize({
    logger: server.log,
    delegate: server.delegate,
  });

  server
    .listen({ port: PORT })
    .then(() => {
      console.log(`Server is running at http://localhost:${PORT}`);
    })
    .catch((error) => {
      console.error('unexpected error', error);
    });
}

main();
