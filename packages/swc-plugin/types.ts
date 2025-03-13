export enum Phase {
  Bundle = 0,
  Runtime = 1,
}

export interface PluginConfig {
  /**
   * The module id.
   */
  id: string;
  /**
   * The module phase.
   */
  phase: Phase;
}
