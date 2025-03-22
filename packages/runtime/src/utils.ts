import type { ModuleRequire } from './types';

const hasOwnProp = Object.prototype.hasOwnProperty;
const defProp = Object.defineProperty;
const copyProps = <T extends object>(
  destination: T,
  source: T,
  except?: string,
): T => {
  for (const key in source) {
    if (
      key !== except &&
      hasOwnProp.call(source, key) &&
      !hasOwnProp.call(destination, key)
    ) {
      defProp(destination, key, {
        enumerable: true,
        get: () => source[key],
      });
    }
  }

  return destination;
};

const toImport = (require: ModuleRequire) => (source: string) =>
  Promise.resolve(require(source));

export { hasOwnProp, defProp, copyProps, toImport };
