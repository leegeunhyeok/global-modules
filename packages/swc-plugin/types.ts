export enum Phase {
  Bundle = 0,
  Runtime = 1,
}

export interface PluginConfig {
  /**
   * The module id.
   */
  id: number;
  /**
   * The module phase.
   */
  phase: Phase;
  /**
   * A map for replacing module sources with IDs.
   */
  paths?: Record<string, number>;
}
