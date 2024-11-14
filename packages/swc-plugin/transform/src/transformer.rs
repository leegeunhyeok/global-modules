use std::mem;

use crate::{constants::DEPS, module_collector::ModuleCollector};
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
}

impl GlobalModuleTransformer {
    pub fn new(id: u64, deps_id: Option<AHashMap<String, u64>>) -> Self {
        GlobalModuleTransformer { id, deps_id }
    }

    fn get_register_expr(
        &self,
        body: Vec<ModuleItem>,
        require_ident: &Ident,
        exports_ident: &Ident,
        deps_ident: &Ident,
    ) -> Vec<ModuleItem> {
        let register_expr = member_expr!(Default::default(), DUMMY_SP, global.__modules.register);
        let scoped_fn = Expr::Fn(FnExpr {
            function: Box::new(Function {
                body: Some(BlockStmt {
                    stmts: body
                        .into_iter()
                        .map(|item| match item.stmt() {
                            Some(stmt) => stmt,
                            _ => panic!("invalid body item"),
                        })
                        .collect::<Vec<Stmt>>(),
                    ..Default::default()
                }),
                params: vec![require_ident.clone().into(), exports_ident.clone().into()],
                ..Default::default()
            }),
            ..Default::default()
        });

        vec![register_expr
            .as_call(
                DUMMY_SP,
                vec![scoped_fn.as_arg(), deps_ident.clone().as_arg()],
            )
            .into_stmt()
            .into()]
    }

    fn get_deps_expr(&self, deps_ident: &Ident) -> Vec<ModuleItem> {
        let obj_lit_expr = Expr::Object(ObjectLit {
            ..Default::default()
        });

        vec![obj_lit_expr
            .into_var_decl(VarDeclKind::Var, deps_ident.clone().into())
            .into()]
    }
}

impl VisitMut for GlobalModuleTransformer {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        let mut imports = vec![];
        let mut exports = vec![];
        let mut body = vec![];
        let mut collector = ModuleCollector::default();

        module.visit_mut_children_with(&mut collector);

        mem::take(&mut module.body)
            .into_iter()
            .for_each(|item| match item {
                ModuleItem::Stmt(_) => body.push(item),
                ModuleItem::ModuleDecl(ref module_decl) => match module_decl {
                    ModuleDecl::Import(_) => imports.push(item),
                    _ => exports.push(item),
                },
            });



        let deps_ident = private_ident!(DEPS);
        let deps_require = collector.get_require_deps_items();
        let scoped_body = self.get_register_expr(
            [deps_require, body].concat(),
            &collector.require_ident,
            &collector.exports_ident,
            &deps_ident,
        );

        let deps_expr = self.get_deps_expr(&deps_ident);

        module.body = [imports, deps_expr, scoped_body, exports].concat();
    }
}
