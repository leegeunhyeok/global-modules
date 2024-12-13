export const __hasOwnProp = Object.prototype.hasOwnProperty;
export const __defProp = Object.defineProperty;

export const __copyProps = <T extends object>(
  destination: T,
  source: T,
  except?: string,
): T => {
  for (const key in source) {
    if (
      key !== except &&
      __hasOwnProp.call(source, key) &&
      !__hasOwnProp.call(destination, key)
    ) {
      __defProp(destination, key, {
        enumerable: true,
        get: () => source[key],
      });
    }
  }

  return destination;
};
