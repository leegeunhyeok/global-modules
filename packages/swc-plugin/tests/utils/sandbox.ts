import * as vm from 'node:vm';

export function evaluateOnSandbox(code: string, context?: vm.Context) {
  const runtimeContext = vm.createContext(context);

  return new vm.Script(code).runInContext(runtimeContext);
}
