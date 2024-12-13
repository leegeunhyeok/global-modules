import { createGlobalModule } from './create-global-module';
import { getGlobalContext } from './get-global-context';

const GLOBAL_MODULE_PROPERTY = '__modules';
const globalContext = getGlobalContext();

if (GLOBAL_MODULE_PROPERTY in globalContext) {
  throw new Error(
    `'${GLOBAL_MODULE_PROPERTY}' property is already defined in the global context.`,
  );
}

Object.defineProperty(globalContext, GLOBAL_MODULE_PROPERTY, {
  value: createGlobalModule(),
});

export type { GlobalModule } from './types';
