use std::mem;

use crate::{module_builder::ModuleBuilder, module_collector::create_collector};
use swc_core::{
    common::{collections::AHashMap, SyntaxContext},
    ecma::{
        ast::*,
        utils::private_ident,
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
};

pub struct GlobalModuleTransformer {
    /// Module ID
    id: String,
    /// Runtime phase flag
    runtime: bool,
    /// Paths
    paths: Option<AHashMap<String, String>>,
    /// Context identifier
    ctx_ident: Ident,
    /// Unresolved context
    unresolved_ctxt: SyntaxContext,
}

impl GlobalModuleTransformer {
    pub fn new(
        id: String,
        runtime: bool,
        paths: Option<AHashMap<String, String>>,
        unresolved_ctxt: SyntaxContext,
    ) -> Self {
        Self {
            id,
            runtime,
            paths,
            unresolved_ctxt,
            ctx_ident: private_ident!("__context"),
        }
    }
}

impl VisitMut for GlobalModuleTransformer {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        let mut collector = create_collector(
            self.unresolved_ctxt,
            self.runtime,
            &self.ctx_ident,
            &self.paths,
        );
        let mut builder = ModuleBuilder::new(&self.ctx_ident);

        module.visit_mut_children_with(&mut collector);
        builder.collect(&mut collector);

        module.body = builder.build_module(&self.id, self.runtime, mem::take(&mut module.body));
    }

    fn visit_mut_script(&mut self, script: &mut Script) {
        let mut collector = create_collector(
            self.unresolved_ctxt,
            self.runtime,
            &self.ctx_ident,
            &self.paths,
        );
        let mut builder = ModuleBuilder::new(&self.ctx_ident);

        script.visit_mut_children_with(&mut collector);
        builder.collect(&mut collector);

        script.body = builder.build_script(&self.id, mem::take(&mut script.body));
    }
}
