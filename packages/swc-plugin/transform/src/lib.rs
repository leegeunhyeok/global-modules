use swc_core::ecma::visit::{as_folder, noop_visit_mut_type, Fold, VisitMut};

pub struct GlobalModuleTransformer {
    #[warn(dead_code)]
    id: u64,
    #[warn(dead_code)]
    dependencies: Option<Vec<u64>>,
}

impl GlobalModuleTransformer {
    fn new(id: u64, dependencies: Option<Vec<u64>>) -> Self {
        GlobalModuleTransformer { id, dependencies }
    }
}

impl VisitMut for GlobalModuleTransformer {
    noop_visit_mut_type!();
}

pub fn global_module(id: u64, dependencies: Option<Vec<u64>>) -> impl VisitMut + Fold {
    as_folder(GlobalModuleTransformer::new(id, dependencies))
}
