export enum Phase {
  Bundle = 0,
  Runtime = 1,
}

export interface PluginConfig {
  id: number;
  phase: Phase;
  dependencyIds?: Record<string, number>;
}
