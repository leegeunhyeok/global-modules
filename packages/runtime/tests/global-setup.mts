import { $ } from 'zx';

function runStep(description: string, step: () => Promise<void>) {
  process.stdout.write(description + ' ');

  function printSymbol(succeed: boolean) {
    process.stdout.write(`${succeed ? '✅' : '❌'}\n\n`);
  }

  return step()
    .then(() => printSymbol(true))
    .catch((error) => {
      printSymbol(false);
      throw error;
    });
}

export default async function () {
  await runStep(
    'Building fixtures...',
    async () => void (await $`node src/__fixtures__/build.mjs`)
  );
}
