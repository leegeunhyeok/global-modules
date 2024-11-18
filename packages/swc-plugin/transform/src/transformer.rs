use std::mem;

use crate::{
    module_collector::{ModuleAst, ModuleCollector},
    utils::ast::wrap_module,
};
use swc_core::{
    common::collections::AHashMap,
    ecma::{
        ast::*,
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
};

pub struct GlobalModuleTransformer {
    pub id: u64,
    pub deps_id: Option<AHashMap<String, u64>>,
}

impl GlobalModuleTransformer {
    pub fn new(id: u64, deps_id: Option<AHashMap<String, u64>>) -> Self {
        GlobalModuleTransformer { id, deps_id }
    }
}

impl VisitMut for GlobalModuleTransformer {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut body = Vec::new();
        let mut collector = ModuleCollector::default();

        module.visit_mut_children_with(&mut collector);

        mem::take(&mut module.body)
            .into_iter()
            .for_each(|item| match item {
                ModuleItem::Stmt(stmt) => body.push(stmt),
                ModuleItem::ModuleDecl(ref module_decl) => match module_decl {
                    ModuleDecl::Import(_) => imports.push(item),
                    _ => exports.push(item),
                },
            });

        let ModuleAst {
            imp_stmts,
            deps_ident,
            deps_decl,
            deps_requires,
            exps_assigns,
            exps_call,
            exps_decl,
        } = collector.get_module_ast();

        let scoped_body = wrap_module(
            [deps_requires, body, exps_assigns, exps_call].concat(),
            &self.id,
            &collector.require_ident,
            &collector.exports_ident,
            &deps_ident,
        );

        module.body = [
            imports,
            imp_stmts,
            deps_decl,
            scoped_body,
            exps_decl,
            exports,
        ]
        .concat();
    }
}
