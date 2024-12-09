use serde::Deserialize;
use swc_core::{
    common::collections::AHashMap,
    ecma::{ast::Program, visit::FoldWith},
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};
use swc_global_module::global_module;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GlobalModuleOptions {
    id: f64,
    phase: f64,
    dependencies: Option<AHashMap<String, f64>>,
}

#[plugin_transform]
pub fn global_module_plugin(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<GlobalModuleOptions>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for @global-modules/swc-plugin"),
    )
    .expect("invalid config for @global-modules/swc-plugin");

    program.fold_with(&mut global_module(
        config.id,
        config.phase,
        config.dependencies,
    ))
}
