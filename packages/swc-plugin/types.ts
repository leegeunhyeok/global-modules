export interface PluginConfig {
  /**
   * The module id.
   */
  id: string;
  /**
   * The flag for transform as runtime module.
   */
  runtime: boolean;
  /**
   * The paths for mapping module sources.
   */
  paths?: Record<string, string>;
}
