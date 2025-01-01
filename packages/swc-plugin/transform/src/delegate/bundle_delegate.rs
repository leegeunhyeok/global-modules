use std::mem;

use helpers::{export_ref_from_named_export, exports_to_ast, get_from_export_decl};
use swc_core::ecma::{ast::*, utils::private_ident};

use super::traits::AstDelegate;
use crate::{
    models::*,
    utils::ast::{
        assign_expr, default_expr_from_default_export_decl, presets::global_module_register_stmt,
        to_binding_module_from_assign_expr, to_binding_module_from_member_expr,
    },
};

pub struct BundleDelegate {
    id: f64,
    ctx_ident: Ident,
    exps: Vec<ExportRef>,
    bindings: Vec<ModuleItem>,
}

impl BundleDelegate {
    pub fn new(id: f64) -> Self {
        Self {
            id,
            ctx_ident: private_ident!("__ctx"),
            exps: Vec::default(),
            bindings: Vec::default(),
        }
    }
}

impl AstDelegate for BundleDelegate {
    fn make_script_body(&mut self, orig_body: Vec<Stmt>) -> Vec<Stmt> {
        let mut new_body = Vec::with_capacity(orig_body.len() + 1);
        new_body.push(global_module_register_stmt(self.id, &self.ctx_ident));
        new_body.extend(orig_body);
        new_body
    }

    fn make_module_body(&mut self, orig_body: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let exps = mem::take(&mut self.exps);
        let bindings = mem::take(&mut self.bindings);
        let ExportsAst {
            leading_body,
            trailing_body,
        } = exports_to_ast(&self.ctx_ident, exps, crate::phase::ModulePhase::Bundle);

        let mut new_body: Vec<ModuleItem> = vec![];
        new_body.push(global_module_register_stmt(self.id, &self.ctx_ident).into());
        new_body.extend(leading_body);
        new_body.extend(orig_body);
        new_body.extend(bindings);
        new_body.extend(trailing_body);

        new_body
    }

    fn import(&mut self, _: &ImportDecl) {}

    fn export_decl(&mut self, export_decl: &ExportDecl) -> ModuleItem {
        let item = get_from_export_decl(export_decl);
        self.exps.push(item.export_ref);
        self.bindings.extend(item.binding_stmts);

        item.export_stmt
    }

    fn export_default_decl(&mut self, export_default_decl: &ExportDefaultDecl) -> ModuleItem {
        let binding_export = BindingExportMember::new("default".into());
        let export_default_expr = default_expr_from_default_export_decl(
            export_default_decl,
            binding_export.bind_ident.clone(),
        );

        self.exps.push(ExportRef::Named(NamedExportRef::new(vec![
            ExportMember::Binding(binding_export),
        ])));

        export_default_expr
    }

    fn export_default_expr(&mut self, export_default_expr: &ExportDefaultExpr) -> Expr {
        let orig_expr = export_default_expr.expr.clone();
        let binding_export = BindingExportMember::new("default".into());
        let binding_assign_expr = assign_expr(binding_export.bind_ident.clone(), *orig_expr).into();

        self.exps.push(ExportRef::Named(NamedExportRef::new(vec![
            ExportMember::Binding(binding_export),
        ])));

        binding_assign_expr
    }

    fn export_named(&mut self, export_named: &NamedExport) {
        self.exps
            .push(export_ref_from_named_export(export_named, &None));
    }

    fn export_all(&mut self, export_all: &ExportAll) {
        self.exps.push(ExportRef::ReExportAll(ReExportAllRef::new(
            export_all.src.value.clone(),
            None,
            None,
        )));
    }

    fn call_expr(&mut self, _: &CallExpr) -> Option<Expr> {
        None
    }

    fn assign_expr(&mut self, assign_expr: &AssignExpr) -> Option<Expr> {
        to_binding_module_from_assign_expr(
            self.ctx_ident.clone(),
            assign_expr,
            crate::phase::ModulePhase::Bundle,
        )
    }

    fn member_expr(&mut self, member_expr: &MemberExpr) -> Option<Expr> {
        to_binding_module_from_member_expr(
            self.ctx_ident.clone(),
            member_expr,
            crate::phase::ModulePhase::Bundle,
        )
    }
}
