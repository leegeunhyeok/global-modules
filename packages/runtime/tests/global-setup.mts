import { $ } from 'zx';

export default async function () {
  await $`node src/__fixtures__/build.mjs`;
}
