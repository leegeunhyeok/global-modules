import { $ } from 'zx';

const date = new Date();
const timestamp =
  date.getFullYear().toString().padStart(4, '0') +
  String(date.getMonth() + 1).padStart(2, '0') +
  String(date.getDate()).padStart(2, '0') +
  String(date.getHours()).padStart(2, '0') +
  String(date.getMinutes()).padStart(2, '0') +
  String(date.getSeconds()).padStart(2, '0');
const version = `0.0.0-dev.${timestamp}`;

await $`git switch -c release/${version}`;
await $`yarn nx release ${version} --tag dev`;
