use swc_core::{
    common::collections::AHashMap,
    ecma::visit::{as_folder, Fold, VisitMut},
};
use transformer::GlobalModuleTransformer;

pub fn global_module(
    id: f64,
    phase: f64,
    deps_id: Option<AHashMap<String, f64>>,
) -> impl VisitMut + Fold {
    as_folder(GlobalModuleTransformer::new(
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
