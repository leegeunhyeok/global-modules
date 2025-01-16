const foo = 'foo';
const bar = 'bar';

export var lazy;

lazy = 'lazy';

// Export named
export { foo, bar, baz as named };

// Export named (with declaration)
export const variable = 1;
export class Class {}
export function func() {}

