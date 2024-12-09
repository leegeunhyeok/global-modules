use std::mem;

use helpers::*;
use swc_core::{
    atoms::Atom,
    ecma::{
        ast::*,
        utils::{private_ident, ExprFactory},
    },
};

use super::traits::AstDelegate;
use crate::{
    models::*,
    utils::{
        ast::{
            assign_expr, expr_from_export_default_decl, global_module_get_ctx_stmt, require_call,
        },
        collections::OHashMap,
    },
};

pub struct RuntimeDelegate {
    id: f64,
    ctx_ident: Ident,
    // `import ... from './foo'`;
    // `require('./foo')`;
    //
    // key: './foo'
    // value: Dep
    deps: OHashMap<Atom, ModuleRef>,
    exps: Vec<ExportRef>,
    bindings: Vec<ModuleItem>,
}

impl RuntimeDelegate {
    pub fn new(id: f64) -> Self {
        Self {
            id,
            ctx_ident: private_ident!("__ctx"),
            deps: OHashMap::default(),
            exps: Vec::default(),
            bindings: Vec::default(),
        }
    }
}

impl AstDelegate for RuntimeDelegate {
    fn make_body(&mut self, orig_body: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let exps = mem::take(&mut self.exps);
        let bindings = mem::take(&mut self.bindings);
        let mut body = Vec::with_capacity(orig_body.len());

        orig_body.into_iter().for_each(|item| match item {
            ModuleItem::Stmt(_) => body.push(item),
            _ => {}
        });

        let deps_items = deps_to_ast(&self.ctx_ident, &self.deps);
        let ExportsAst {
            pre_body,
            post_body,
        } = exports_to_ast(&self.ctx_ident, exps, crate::phase::ModulePhase::Runtime);

        let mut new_body: Vec<ModuleItem> = vec![];
        new_body.push(global_module_get_ctx_stmt(self.id, &self.ctx_ident));
        new_body.extend(deps_items);
        new_body.extend(pre_body);
        new_body.extend(body);
        new_body.extend(bindings);
        new_body.extend(post_body);

        new_body
    }

    fn import(&mut self, import_decl: &ImportDecl) {
        let members = to_import_members(&import_decl.specifiers);
        let src = &import_decl.src.value;

        if let Some(mod_ref) = self.deps.get_mut(&src) {
            mod_ref.members.extend(members.into_iter());
        } else {
            self.deps.insert(src, ModuleRef::new(members));
        }
    }

    fn export_decl(&mut self, export_decl: &ExportDecl) -> ModuleItem {
        let item = get_from_export_decl(export_decl);
        self.exps.push(item.export_ref);
        self.bindings.push(item.binding_stmt);

        item.export_stmt
    }

    fn export_default_decl(&mut self, export_default_decl: &ExportDefaultDecl) -> ModuleItem {
        let binding_export = BindingExportMember::new("default".into());
        let assign_expr =
            expr_from_export_default_decl(export_default_decl, binding_export.bind_ident.clone())
                .into_stmt()
                .into();

        self.exps.push(ExportRef::Named(NamedExportRef::new(vec![
            ExportMember::Binding(binding_export),
        ])));

        assign_expr
    }

    fn export_default_expr(&mut self, export_default_expr: &ExportDefaultExpr) -> Option<Expr> {
        let orig_expr = (*export_default_expr.expr).clone();
        let binding_export = BindingExportMember::new("default".into());
        let binding_assign_expr = assign_expr(&binding_export.bind_ident, orig_expr).into();

        self.exps.push(ExportRef::Named(NamedExportRef::new(vec![
            ExportMember::Binding(binding_export),
        ])));

        Some(binding_assign_expr)
    }

    fn export_named(&mut self, export_named: &NamedExport) {
        let export_ref = get_from_export_named(export_named);
        self.exps.push(export_ref);
    }

    fn export_all(&mut self, export_all: &ExportAll) {
        self.exps.push(ExportRef::ReExportAll(ReExportAllRef::new(
            &export_all.src.value,
            None,
        )));
    }

    fn call_expr(&mut self, call_expr: &CallExpr) -> Option<Expr> {
        match call_expr {
            // Replace CommonJS requires.
            //
            // ```js
            // require('...');
            // ```
            CallExpr {
                args,
                callee: Callee::Expr(callee_expr),
                type_args: None,
                ..
            } if args.len() == 1 && callee_expr.is_ident_ref_to("require") => {
                let src = args.get(0).unwrap();

                match &*src.expr {
                    // The first argument of the `require` function must be a string type only.
                    Expr::Lit(Lit::Str(str)) => {
                        return Some(require_call(&self.ctx_ident, &str.value))
                    }
                    _ => panic!("invalid `require` call expression"),
                }
            }
            // Replace ESModule's dynamic imports.
            //
            // ```js
            // import('...', {});
            // ```
            CallExpr {
                args,
                callee: Callee::Import(_),
                type_args: None,
                ..
            } => {
                let src = args.get(0).expect("invalid dynamic import call");

                match &*src.expr {
                    // The first argument of the `import` function must be a string type only.
                    Expr::Lit(Lit::Str(str)) => {
                        return Some(require_call(&self.ctx_ident, &str.value))
                    }
                    _ => panic!("unsupported dynamic import usage"),
                }
            }
            _ => return None,
        }
    }

    fn assign_expr(&mut self, _: &AssignExpr) -> Option<Expr> {
        unimplemented!("TODO");
    }
}
