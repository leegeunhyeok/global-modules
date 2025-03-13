use std::mem;

use crate::{
    module_builder::ModuleBuilder, module_collector::create_collector, phase::ModulePhase,
};
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
    /// Module phase
    phase: ModulePhase,
    /// Context identifier
    ctx_ident: Ident,
    /// Dependencies identifier
    deps_ident: Ident,
    /// Unresolved context
    unresolved_ctxt: SyntaxContext,
}

impl GlobalModuleTransformer {
    pub fn new(id: String, phase: ModulePhase, unresolved_ctxt: SyntaxContext) -> Self {
        Self {
            id,
            phase,
            unresolved_ctxt,
            ctx_ident: private_ident!("__context"),
            deps_ident: private_ident!("__deps"),
        }
    }
}

impl VisitMut for GlobalModuleTransformer {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        let mut collector = create_collector(self.unresolved_ctxt, &self.ctx_ident);
        let mut builder = ModuleBuilder::new(&self.ctx_ident, &self.deps_ident);

        module.visit_mut_children_with(&mut collector);
        builder.collect_module_body(&mut collector, mem::take(&mut module.body));

        module.body = builder.build_module(&self.id, self.phase);
    }

    fn visit_mut_script(&mut self, script: &mut Script) {
        let mut collector = create_collector(self.unresolved_ctxt, &self.ctx_ident);
        let mut builder = ModuleBuilder::new(&self.ctx_ident, &self.deps_ident);

        script.visit_mut_children_with(&mut collector);
        builder.collect_script_body(&mut collector, mem::take(&mut script.body));

        script.body = builder.build_script(&self.id, self.phase);
    }
}
