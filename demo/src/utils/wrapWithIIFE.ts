export function wrapWithIIFE(source: string) {
  return `(function() {
    ${source}
  })();`;
}
