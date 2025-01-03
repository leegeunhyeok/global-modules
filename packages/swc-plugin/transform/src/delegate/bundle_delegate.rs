use std::mem;

use helpers::{export_ref_from_named_export, exports_to_ast, get_from_export_decl};
use swc_core::{
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::{private_ident, ExprFactory},
    },
};

use super::traits::AstDelegate;
use crate::{
    models::*,
    utils::ast::{
        assign_expr, get_expr_from_default_decl, get_ident_from_default_decl, into_decl,
        presets::global_module_register_stmt, to_binding_module_from_assign_expr,
        to_binding_module_from_member_expr,
    },
};

pub struct BundleDelegate {
    id: f64,
    ctx_ident: Ident,
    exps: Vec<ExportRef>,
    hoisted_decls: Vec<ModuleItem>,
    bindings: Vec<ModuleItem>,
}

impl BundleDelegate {
    pub fn new(id: f64) -> Self {
        Self {
            id,
            ctx_ident: private_ident!("__ctx"),
            exps: Vec::default(),
            hoisted_decls: Vec::default(),
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
        let hoisted_decls = mem::take(&mut self.hoisted_decls);
        let bindings = mem::take(&mut self.bindings);
        let ExportsAst {
            leading_body,
            trailing_body,
        } = exports_to_ast(&self.ctx_ident, exps, crate::phase::ModulePhase::Bundle);

        let mut stmts = Vec::with_capacity(orig_body.len());
        let mut imports = Vec::with_capacity(orig_body.len());
        let mut exports = Vec::with_capacity(orig_body.len());
        let mut new_body: Vec<ModuleItem> = Vec::with_capacity(
            1 + leading_body.len() + orig_body.len() + bindings.len() + trailing_body.len(),
        );
        new_body.push(global_module_register_stmt(self.id, &self.ctx_ident).into());
        new_body.extend(leading_body);
        new_body.extend(hoisted_decls);
        new_body.extend(orig_body);
        new_body.extend(bindings);
        new_body.extend(trailing_body);

        new_body.into_iter().for_each(|item| match item {
            ModuleItem::ModuleDecl(ref module_decl) => match module_decl {
                ModuleDecl::Import(_) => imports.push(item),
                _ => exports.push(item),
            },
            ModuleItem::Stmt(_) => stmts.push(item),
        });

        [imports, stmts, exports].concat()
    }

    fn import(&mut self, _: &ImportDecl) {}

    fn export_decl(&mut self, export_decl: &ExportDecl) -> ModuleItem {
        let item = get_from_export_decl(export_decl);
        self.exps.push(item.export_ref);
        self.hoisted_decls.push(export_decl.decl.clone().into());
        self.bindings.push(item.binding_stmt);

        item.export_stmt
    }

    fn export_default_decl(
        &mut self,
        export_default_decl: &ExportDefaultDecl,
    ) -> Option<ModuleItem> {
        let ident = get_ident_from_default_decl(&export_default_decl.decl);
        let binding_export = BindingExportMember::new("default".into());
        let binding_export_stmt =
            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
                span: DUMMY_SP,
                expr: Box::new(binding_export.bind_ident.clone().into()),
            }));

        match ident {
            Some(ident) => {
                self.hoisted_decls
                    .push(into_decl(&export_default_decl.decl).into());
                self.bindings.push(
                    assign_expr(binding_export.bind_ident.clone(), ident.into())
                        .into_stmt()
                        .into(),
                );
            }
            None => self.bindings.push(
                assign_expr(
                    binding_export.bind_ident.clone(),
                    get_expr_from_default_decl(&export_default_decl.decl).into(),
                )
                .into_stmt()
                .into(),
            ),
        };

        self.exps.push(ExportRef::Named(NamedExportRef::new(vec![
            ExportMember::Binding(binding_export),
        ])));

        binding_export_stmt.into()
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
