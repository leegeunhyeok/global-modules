import { Bundler } from '../bundler';
import { server } from './server';

const PORT = 3000;

async function main() {
  await Bundler.getInstance().initialize({ logger: server.log });

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
