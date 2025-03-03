use swc_core::{
    common::{collections::AHashMap, SyntaxContext},
    ecma::{
        ast::Pass,
        visit::{visit_mut_pass, VisitMut},
    },
};
use transformer::GlobalModuleTransformer;

pub fn global_modules(
    id: String,
    phase: f64,
    paths: Option<AHashMap<String, String>>,
    unresolved_ctxt: SyntaxContext,
) -> impl VisitMut + Pass {
    visit_mut_pass(GlobalModuleTransformer::new(
        id,
        (phase as u32).into(),
        paths,
        unresolved_ctxt,
    ))
}

mod models;
mod module_collector;
mod phase;
mod transformer;
mod utils;
