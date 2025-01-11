export function registerHotModule(code: string, id: number) {
  return [
    code,
    `
    if (window.__hot) {
      const context = window.__hot.register(${id});
      context.dispose(() => console.log('[HMR] Disposed', ${id}));
      context.accept(() => {
        console.log('[HMR] Accepted ::', ${id});
        window.__hot.reactRefresh.performReactRefresh();
      });
    }
    `,
  ].join('\n');
}
