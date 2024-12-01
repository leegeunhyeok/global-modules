use swc_core::{
    common::collections::AHashMap,
    ecma::visit::{as_folder, Fold, VisitMut},
};
use transformer::GlobalModuleTransformer;

pub fn global_module(
    id: u32,
    phase: u32,
    deps_id: Option<AHashMap<String, u32>>,
) -> impl VisitMut + Fold {
    as_folder(GlobalModuleTransformer::new(id, phase.into(), deps_id))
}

mod constants;
mod model_helpers;
mod models;
mod phase;
mod transformer;
mod utils;
