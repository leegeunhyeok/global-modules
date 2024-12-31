use std::mem;

use helpers::*;
use swc_core::{
    atoms::Atom,
    common::collections::AHashMap,
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
            assign_expr, expr_from_export_default_decl, get_src_lit,
            presets::{global_module_get_ctx_stmt, require_call},
            to_binding_module_from_assign_expr, to_binding_module_from_member_expr,
        },
        collections::OHashMap,
    },
};

pub struct RuntimeDelegate {
    id: f64,
    paths: Option<AHashMap<String, f64>>,
    ctx_ident: Ident,
    deps: OHashMap<Atom, ModuleRef>,
    exps: Vec<ExportRef>,
    bindings: Vec<ModuleItem>,
}

impl RuntimeDelegate {
    pub fn new(id: f64, paths: Option<AHashMap<String, f64>>) -> Self {
        Self {
            id,
            paths,
            ctx_ident: private_ident!("__ctx"),
            deps: OHashMap::default(),
            exps: Vec::default(),
            bindings: Vec::default(),
        }
    }
}

impl AstDelegate for RuntimeDelegate {
    fn make_script_body(&mut self, orig_body: Vec<Stmt>) -> Vec<Stmt> {
        let mut new_body = Vec::with_capacity(orig_body.len() + 1);
        new_body.push(global_module_get_ctx_stmt(self.id, &self.ctx_ident));
        new_body.extend(orig_body);
        new_body
    }

    fn make_module_body(&mut self, mut orig_body: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let exps = mem::take(&mut self.exps);
        let bindings = mem::take(&mut self.bindings);

        orig_body.retain(|item| item.is_stmt());

        let deps_items = deps_to_ast(&self.ctx_ident, &self.deps, &self.paths);
        let ExportsAst {
            leading_body,
            trailing_body,
        } = exports_to_ast(&self.ctx_ident, exps, crate::phase::ModulePhase::Runtime);
        let total_item_count = deps_items.len()
            + leading_body.len()
            + orig_body.len()
            + bindings.len()
            + trailing_body.len()
            + 1;

        let mut new_body: Vec<ModuleItem> = Vec::with_capacity(total_item_count);
        new_body.push(global_module_get_ctx_stmt(self.id, &self.ctx_ident).into());
        new_body.extend(deps_items);
        new_body.extend(leading_body);
        new_body.extend(orig_body);
        new_body.extend(bindings);
        new_body.extend(trailing_body);
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
        let orig_expr = export_default_expr.expr.clone();
        let binding_export = BindingExportMember::new("default".into());
        let binding_assign_expr = assign_expr(binding_export.bind_ident.clone(), *orig_expr).into();

        self.exps.push(ExportRef::Named(NamedExportRef::new(vec![
            ExportMember::Binding(binding_export),
        ])));

        Some(binding_assign_expr)
    }

    fn export_named(&mut self, export_named: &NamedExport) {
        self.exps
            .push(export_ref_from_named_export(export_named, &self.paths));
    }

    fn export_all(&mut self, export_all: &ExportAll) {
        let src = export_all.src.value.clone();
        let id = self
            .paths
            .as_ref()
            .and_then(|paths| paths.get(src.as_str()));

        self.exps.push(ExportRef::ReExportAll(ReExportAllRef::new(
            src,
            id.copied(),
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
                    Expr::Lit(lit) => {
                        return Some(require_call(
                            self.ctx_ident.clone(),
                            get_src_lit(lit, &self.paths),
                        ));
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
                    Expr::Lit(lit) => {
                        return Some(require_call(
                            self.ctx_ident.clone(),
                            get_src_lit(lit, &self.paths),
                        ));
                    }
                    _ => panic!("unsupported dynamic import usage"),
                }
            }
            _ => return None,
        }
    }

    fn assign_expr(&mut self, assign_expr: &AssignExpr) -> Option<Expr> {
        to_binding_module_from_assign_expr(
            self.ctx_ident.clone(),
            assign_expr,
            crate::phase::ModulePhase::Runtime,
        )
    }

    fn member_expr(&mut self, member_expr: &MemberExpr) -> Option<Expr> {
        to_binding_module_from_member_expr(
            self.ctx_ident.clone(),
            member_expr,
            crate::phase::ModulePhase::Runtime,
        )
    }
}
