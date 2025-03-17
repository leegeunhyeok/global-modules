export function registerHotModule(code: string, id: string) {
  const idString = JSON.stringify(id);
  return [
    code,
    `
    if (global.__modules) {
      const module = global.__modules.getModule(${idString});
      module.meta = {};
      module.meta.hot = window.__modules.hot(${idString});
      module.meta.hot.dispose(() => console.log('[HMR] Disposed ::', ${idString}));
      module.meta.hot.accept(() => {
        console.log('[HMR] Accepted ::', ${idString});
        // window.$$reactRefresh$$.performReactRefresh();
      });
    }
    `,
  ].join('\n');
}

export function applyModule(
  id: string,
  dependencyMap?: Record<string, string>,
) {
  const idString = JSON.stringify(id);
  return `
  if (global.__modules) {
    global.__modules.apply(${idString}, ${dependencyMap ? JSON.stringify(dependencyMap) : 'undefined'});
  }
  `;
}
