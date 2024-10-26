use serde::Deserialize;
use swc_core::{
    ecma::{ast::Program, visit::FoldWith},
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};
use swc_global_module::global_module;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GlobalModuleOptions {
    id: Option<u64>,
    dependencies: Option<Vec<u64>>,
}

#[plugin_transform]
pub fn global_module_plugin(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<GlobalModuleOptions>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for @global-module/swc-plugin"),
    )
    .expect("invalid config for @global-module/swc-plugin");

    program.fold_with(&mut global_module(
        config.id.expect("id is required"),
        config.dependencies,
    ))
}
