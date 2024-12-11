use std::mem;

use helpers::{exports_to_ast, get_from_export_decl, to_export_ref};
use swc_core::ecma::{ast::*, utils::private_ident};

use super::traits::AstDelegate;
use crate::{
    models::*,
    utils::ast::{
        assign_expr, default_expr_from_default_export_decl, presets::global_module_register_stmt,
        to_binding_module_from_assign_expr, to_binding_module_from_member_expr,
    },
};

pub struct RegisterDelegate {
    id: f64,
    ctx_ident: Ident,
    exps: Vec<ExportRef>,
    bindings: Vec<ModuleItem>,
}

impl RegisterDelegate {
    pub fn new(id: f64) -> Self {
        Self {
            id,
            ctx_ident: private_ident!("__ctx"),
            exps: Vec::default(),
            bindings: Vec::default(),
        }
    }
}

impl AstDelegate for RegisterDelegate {
    fn make_body(&mut self, orig_body: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let size = orig_body.len();
        let exps = mem::take(&mut self.exps);
        let bindings = mem::take(&mut self.bindings);
        let mut imports = Vec::with_capacity(size);
        let mut exports = Vec::with_capacity(size);
        let mut body = Vec::with_capacity(size);

        orig_body.into_iter().for_each(|item| match item {
            ModuleItem::Stmt(_) => body.push(item),
            ModuleItem::ModuleDecl(ref module_decl) => match module_decl {
                ModuleDecl::Import(_) => imports.push(item),
                _ => exports.push(item),
            },
        });

        let ExportsAst {
            pre_body,
            post_body,
        } = exports_to_ast(&self.ctx_ident, exps, crate::phase::ModulePhase::Register);

        let mut new_body: Vec<ModuleItem> = vec![];
        new_body.push(global_module_register_stmt(self.id, &self.ctx_ident));
        new_body.extend(imports);
        new_body.extend(pre_body);
        new_body.extend(body);
        new_body.extend(bindings);
        new_body.extend(post_body);
        new_body.extend(exports);

        new_body
    }

    fn import(&mut self, _: &ImportDecl) {}

    fn export_decl(&mut self, export_decl: &ExportDecl) -> ModuleItem {
        let item = get_from_export_decl(export_decl);
        self.exps.push(item.export_ref);
        self.bindings.push(item.binding_stmt);

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
        let export_ref = to_export_ref(export_named);
        self.exps.push(export_ref);
    }

    fn export_all(&mut self, export_all: &ExportAll) {
        self.exps.push(ExportRef::ReExportAll(ReExportAllRef::new(
            &export_all.src.value,
            None,
        )));
    }

    fn call_expr(&mut self, _: &CallExpr) -> Option<Expr> {
        None
    }

    fn assign_expr(&mut self, assign_expr: &AssignExpr) -> Option<Expr> {
        to_binding_module_from_assign_expr(
            &self.ctx_ident,
            assign_expr,
            crate::phase::ModulePhase::Register,
        )
    }

    fn member_expr(&mut self, member_expr: &MemberExpr) -> Option<Expr> {
        to_binding_module_from_member_expr(
            &self.ctx_ident,
            member_expr,
            crate::phase::ModulePhase::Register,
        )
    }
}
