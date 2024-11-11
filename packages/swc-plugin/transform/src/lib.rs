use swc_core::{
    common::collections::AHashMap,
    ecma::visit::{as_folder, Fold, VisitMut},
};
use transformer::GlobalModuleTransformer;

pub fn global_module(id: u64, deps_id: Option<AHashMap<String, u64>>) -> impl VisitMut + Fold {
    as_folder(GlobalModuleTransformer::new(id, deps_id))
}

mod constants;
mod expr_utils;
mod module_collector;
mod transformer;
