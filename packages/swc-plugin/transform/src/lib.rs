use swc_core::{
    common::collections::AHashMap,
    ecma::{
        ast::Pass,
        visit::{visit_mut_pass, VisitMut},
    },
};
use transformer::GlobalModuleTransformer;

pub fn global_modules(
    id: f64,
    phase: f64,
    deps_id: Option<AHashMap<String, f64>>,
) -> impl VisitMut + Pass {
    visit_mut_pass(GlobalModuleTransformer::new(
        id,
        (phase as u32).into(),
        deps_id,
    ))
}

mod delegate;
mod models;
mod phase;
mod transformer;
mod utils;
