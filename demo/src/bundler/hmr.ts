export function registerHotModule(code: string, id: number) {
  return [
    code,
    `
    if (global.__modules) {
      const context = global.__modules.getContext(${id});
      context.dispose = () => console.log('[HMR] Disposed', ${id});
      context.accept = () => {
        console.log('[HMR] Accepted ::', ${id});
        window.$$reactRefresh$$.performReactRefresh();
      };
    }
    `,
  ].join('\n');
}
