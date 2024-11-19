use std::mem;

use crate::{
    constants::*,
    module_collector::{ModuleAst, ModuleCollector},
    utils::ast::num_lit_expr,
};
use swc_core::{
    common::{collections::AHashMap, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{member_expr, private_ident, ExprFactory},
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
};

pub struct GlobalModuleTransformer {
    pub id: u64,
    pub deps_id: Option<AHashMap<String, u64>>,
    exports_ident: Ident,
    require_ident: Ident,
}

impl GlobalModuleTransformer {
    pub fn new(id: u64, deps_id: Option<AHashMap<String, u64>>) -> Self {
        GlobalModuleTransformer {
            id,
            deps_id,
            exports_ident: private_ident!(EXPORTS_ARG),
            require_ident: private_ident!(REQUIRE_ARG),
        }
    }

    /// Wraps the module body with an expression to register it as a global module.
    ///
    /// ```js
    /// global.__module.register(function (__require, __exports) {
    ///   /* body */
    /// }, id, __deps);
    /// ```
    fn as_global_module(&self, body: Vec<Stmt>, deps_ident: Ident) -> Vec<ModuleItem> {
        let register_expr = member_expr!(Default::default(), DUMMY_SP, global.__modules.register);
        let scoped_fn = Expr::Fn(FnExpr {
            function: Box::new(Function {
                body: Some(BlockStmt {
                    stmts: body,
                    ..Default::default()
                }),
                params: vec![
                    self.require_ident.clone().into(),
                    self.exports_ident.clone().into(),
                ],
                ..Default::default()
            }),
            ..Default::default()
        });

        vec![register_expr
            .as_call(
                DUMMY_SP,
                vec![
                    scoped_fn.as_arg(),
                    num_lit_expr(self.id as f64).as_arg(),
                    deps_ident.as_arg(),
                ],
            )
            .into_stmt()
            .into()]
    }
}

impl VisitMut for GlobalModuleTransformer {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut body = Vec::new();
        let mut collector = ModuleCollector::new(&self.exports_ident, &self.require_ident);

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
            import_stmts,
            deps_ident,
            deps_decl,
            deps_requires,
            export_assigns,
            export_call,
            export_decls,
        } = collector.get_module_ast();

        let scoped_body = self.as_global_module(
            [deps_requires, body, export_assigns, export_call].concat(),
            deps_ident,
        );

        module.body = [
            imports,
            import_stmts,
            deps_decl,
            scoped_body,
            export_decls,
            exports,
        ]
        .concat();
    }
}
