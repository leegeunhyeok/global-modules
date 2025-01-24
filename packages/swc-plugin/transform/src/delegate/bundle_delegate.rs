use std::mem;

use helpers::{export_ref_from_named_export, exports_to_ast, get_from_export_decl};
use swc_core::{
    common::{SyntaxContext, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{private_ident, ExprFactory},
    },
};

use super::traits::AstDelegate;
use crate::{
    models::*,
    utils::ast::presets::global_module_register_stmt,
    utils::ast::{
        assign_expr, assign_member, get_expr_from_default_decl, get_ident_from_default_decl,
        into_decl, to_binding_module_from_assign_expr, to_binding_module_from_member_expr,
    },
};

pub struct BundleDelegate {
    unresolved_ctxt: SyntaxContext,
    id: String,
    ctx_ident: Ident,
    exports: Vec<ExportRef>,
    hoisted_decls: Vec<ModuleItem>,
    export_decls: Vec<ModuleItem>,
    bindings: Vec<ModuleItem>,
}

impl BundleDelegate {
    pub fn new(id: String, unresolved_ctxt: SyntaxContext) -> Self {
        Self {
            unresolved_ctxt,
            id,
            ctx_ident: private_ident!("__ctx"),
            exports: Vec::default(),
            hoisted_decls: Vec::default(),
            export_decls: Vec::default(),
            bindings: Vec::default(),
        }
    }
}

impl AstDelegate for BundleDelegate {
    fn make_script_body(&mut self, orig_body: Vec<Stmt>) -> Vec<Stmt> {
        let mut new_body = Vec::with_capacity(orig_body.len() + 1);
        new_body.push(global_module_register_stmt(&self.id, &self.ctx_ident));
        new_body.extend(orig_body);
        new_body
    }

    fn make_module_body(&mut self, orig_body: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let exports = mem::take(&mut self.exports);
        let hoisted_decls = mem::take(&mut self.hoisted_decls);
        let export_decls = mem::take(&mut self.export_decls);
        let bindings = mem::take(&mut self.bindings);
        let ExportsAst {
            leading_body,
            trailing_body,
        } = exports_to_ast(&self.ctx_ident, exports, crate::phase::ModulePhase::Bundle);

        let total_capacity = 1
            + leading_body.len()
            + hoisted_decls.len()
            + orig_body.len()
            + export_decls.len()
            + bindings.len()
            + trailing_body.len();

        let mut stmts = Vec::with_capacity(orig_body.len());
        let mut imports = Vec::with_capacity(orig_body.len());
        let mut exports = Vec::with_capacity(orig_body.len());
        let mut new_body: Vec<ModuleItem> = Vec::with_capacity(total_capacity);

        new_body.push(global_module_register_stmt(&self.id, &self.ctx_ident).into());
        new_body.extend(leading_body);
        new_body.extend(hoisted_decls);
        new_body.extend(orig_body);
        new_body.extend(export_decls);
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

    fn import(&mut self, _: &mut ImportDecl) {}

    fn export_decl(&mut self, export_decl: &mut ExportDecl) -> ModuleItem {
        let item = get_from_export_decl(export_decl);
        self.exports.push(item.export_ref);
        self.export_decls.push(item.export_stmt);
        self.bindings.push(item.binding_stmt);

        export_decl.decl.clone().into()
    }

    fn export_default_decl(
        &mut self,
        export_default_decl: &mut ExportDefaultDecl,
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

        self.exports.push(ExportRef::Named(NamedExportRef::new(vec![
            ExportMember::Binding(binding_export),
        ])));

        binding_export_stmt.into()
    }

    fn export_default_expr(
        &mut self,
        export_default_expr: &mut ExportDefaultExpr,
    ) -> Option<ModuleItem> {
        let binding_export = BindingExportMember::new("default".into());
        let binding_assign_expr = assign_expr(
            binding_export.bind_ident.clone(),
            *export_default_expr.expr.clone(),
        );

        self.exports.push(ExportRef::Named(NamedExportRef::new(vec![
            ExportMember::Binding(binding_export),
        ])));

        *export_default_expr.expr = binding_assign_expr.into();

        None
    }

    fn export_named(&mut self, export_named: &mut NamedExport) {
        self.exports
            .push(export_ref_from_named_export(export_named, &None));
    }

    fn export_all(&mut self, export_all: &mut ExportAll) {
        self.exports
            .push(ExportRef::ReExportAll(ReExportAllRef::new(
                export_all.src.value.clone(),
                None,
                None,
            )));
    }

    fn call_expr(&mut self, _: &mut CallExpr) -> Option<Expr> {
        None
    }

    fn assign_expr(&mut self, assign_expr: &mut AssignExpr) -> Option<Expr> {
        if let Some(new_assign_expr) = to_binding_module_from_assign_expr(
            self.ctx_ident.clone(),
            assign_expr,
            self.unresolved_ctxt,
        ) {
            Some(new_assign_expr.make_assign_to(AssignOp::Assign, assign_expr.left.clone()))
        } else {
            None
        }
    }

    fn member_expr(&mut self, member_expr: &mut MemberExpr) -> Option<Expr> {
        if let Some(new_member_expr) = to_binding_module_from_member_expr(
            self.ctx_ident.clone(),
            member_expr,
            self.unresolved_ctxt,
        ) {
            Some(assign_member(member_expr.clone(), new_member_expr.into()))
        } else {
            None
        }
    }
}
