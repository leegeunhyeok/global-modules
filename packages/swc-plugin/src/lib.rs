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
    id: u64,
    dependencies: Option<AHashMap<String, u64>>,
}

#[plugin_transform]
pub fn global_module_plugin(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<GlobalModuleOptions>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for @global-module/swc-plugin"),
    )
    .expect("invalid config for @global-module/swc-plugin");

    program.fold_with(&mut global_module(config.id, config.dependencies))
}
