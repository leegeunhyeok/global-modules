/* eslint-disable no-console -- allow */
import { $ } from 'zx';

function runStep(
  description: string,
  step: () => Promise<void>,
): Promise<void> {
  process.stdout.write(`${description} `);

  function printSymbol(succeed: boolean): void {
    process.stdout.write(`${succeed ? '✅' : '❌'}\n`);
  }

  return step()
    .then(() => printSymbol(true))
    .catch((error: unknown) => {
      printSymbol(false);
      throw error;
    });
}

// eslint-disable-next-line import/no-default-export -- allow default export
export default async function setup(): Promise<void> {
  await runStep(
    'Building runtime module...',
    async () => void (await $`yarn build`),
  );
  console.log(''); // Print trailing space before run test.
}
