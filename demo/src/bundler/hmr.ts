export function registerHotModule(code: string, id: number) {
  return [
    code,
    `
    if (global.__modules) {
      const context = global.__modules.getContext(${id});
      context.hot = window.__modules.hot(${id});
      context.hot.dispose(() => console.log('[HMR] Disposed', ${id}));
      context.hot.accept(() => {
        console.log('[HMR] Accepted ::', ${id});
        window.$$reactRefresh$$.performReactRefresh();
      });
    }
    `,
  ].join('\n');
}
