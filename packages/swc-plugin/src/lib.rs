use serde::Deserialize;
use swc_core::{
    common::collections::AHashMap,
    ecma::{ast::Program, visit::VisitMutWith},
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};
use swc_global_modules::global_modules;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GlobalModuleConfig {
    id: f64,
    phase: f64,
    paths: Option<AHashMap<String, f64>>,
}

#[plugin_transform]
pub fn global_modules_plugin(
    mut program: Program,
    metadata: TransformPluginProgramMetadata,
) -> Program {
    let config = serde_json::from_str::<GlobalModuleConfig>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for @global-modules/swc-plugin"),
    )
    .expect("invalid config for @global-modules/swc-plugin");

    program.visit_mut_with(&mut global_modules(config.id, config.phase, config.paths));

    program
}
