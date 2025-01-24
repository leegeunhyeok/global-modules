export function registerHotModule(code: string, id: string) {
  const idString = JSON.stringify(id);
  return [
    code,
    `
    if (global.__modules) {
      const context = global.__modules.getContext(${idString});
      context.hot = window.__modules.hot(${idString});
      context.hot.dispose(() => console.log('[HMR] Disposed', ${idString}));
      context.hot.accept(() => {
        console.log('[HMR] Accepted ::', ${idString});
        window.$$reactRefresh$$.performReactRefresh();
      });
    }
    `,
  ].join('\n');
}
