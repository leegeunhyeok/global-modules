import { server } from './server';

const PORT = 3000;

server
  .listen({ port: PORT })
  .then(() => {
    console.log(`Server is running at http://localhost:${PORT}`);
  })
  .catch((error) => {
    console.error('unexpected error', error);
  });
