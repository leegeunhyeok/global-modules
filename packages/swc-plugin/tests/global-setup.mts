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

export default async function setup(): Promise<void> {
  await runStep(
    'Building swc-plugin...',
    async () => void (await $`yarn build`),
  );
  console.log(''); // Print trailing space before run test.
}
