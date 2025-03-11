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
    id: String,
    phase: ModulePhase,
    paths: Option<AHashMap<String, String>>,
    unresolved_ctxt: SyntaxContext,
    ctx_ident: Ident,
}

impl GlobalModuleTransformer {
    pub fn new(
        id: String,
        phase: ModulePhase,
        paths: Option<AHashMap<String, String>>,
        unresolved_ctxt: SyntaxContext,
    ) -> Self {
        Self {
            id,
            phase,
            paths,
            unresolved_ctxt,
            ctx_ident: private_ident!("__context"),
        }
    }
}

impl VisitMut for GlobalModuleTransformer {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        let mut collector = create_collector(self.unresolved_ctxt, &self.ctx_ident, &self.paths);
        let mut builder = ModuleBuilder::new();

        module.visit_mut_children_with(&mut collector);
        builder.collect_module_body(&mut collector, mem::take(&mut module.body));

        module.body = builder.build(&self.id);
    }

    fn visit_mut_script(&mut self, script: &mut Script) {
        let mut collector = create_collector(self.unresolved_ctxt, &self.ctx_ident, &self.paths);
        let mut builder = ModuleBuilder::new();

        script.visit_mut_children_with(&mut collector);
        builder.collect_script_body(&mut collector, mem::take(&mut script.body));

        // TODO: build_script
        script.body = builder
            .build(&self.id)
            .into_iter()
            .filter_map(|item| match item {
                ModuleItem::Stmt(stmt) => Some(stmt),
                _ => None,
            })
            .collect();
    }
}
