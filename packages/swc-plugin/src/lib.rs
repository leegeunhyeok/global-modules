use serde::Deserialize;
use swc_core::{
    common::SyntaxContext,
    ecma::ast::Program,
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};
use swc_global_modules::global_modules;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GlobalModuleConfig {
    id: String,
    runtime: bool,
}

#[plugin_transform]
pub fn global_modules_plugin(
    program: Program,
    metadata: TransformPluginProgramMetadata,
) -> Program {
    let config = serde_json::from_str::<GlobalModuleConfig>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for @global-modules/swc-plugin"),
    )
    .expect("invalid config for @global-modules/swc-plugin");

    program.apply(&mut global_modules(
        config.id,
        config.runtime,
        SyntaxContext::empty().apply_mark(metadata.unresolved_mark),
    ))
}
