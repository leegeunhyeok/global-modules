/** @type {import('@yarnpkg/types')} */
const { defineConfig } = require('@yarnpkg/types');

module.exports = defineConfig({
  async constraints({ Yarn }) {
    for (const dep of Yarn.dependencies({ ident: 'typescript' })) {
      dep.update('5.6.2');
    }

    for (const dep of Yarn.dependencies({ ident: '@types/node' })) {
      dep.update('^22');
    }
  },
});
