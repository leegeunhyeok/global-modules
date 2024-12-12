export enum Phase {
  Register = 0,
  Runtime = 1,
}

export interface PluginConfig {
  id: number;
  phase: Phase;
  dependencies?: Record<string, number>;
}
