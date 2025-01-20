use std::mem;

use helpers::*;
use swc_core::{
    atoms::Atom,
    common::{collections::AHashMap, Spanned},
    ecma::{
        ast::*,
        utils::{private_ident, ExprFactory},
    },
    plugin::errors::HANDLER,
};

use super::traits::AstDelegate;
use crate::{
    models::*,
    utils::{
        ast::{
            assign_expr, get_expr_from_default_decl, get_ident_from_default_decl, get_src_lit,
            into_decl,
            presets::{ctx_reset_call, global_module_get_ctx_stmt, require_call},
            to_binding_module_from_assign_expr, to_binding_module_from_member_expr,
        },
        collections::OHashMap,
    },
};

pub struct RuntimeDelegate {
    id: String,
    paths: Option<AHashMap<String, String>>,
    ctx_ident: Ident,
    deps: OHashMap<Atom, ModuleRef>,
    exports: Vec<ExportRef>,
    hoisted_decls: Vec<ModuleItem>,
    export_decls: Vec<ModuleItem>,
    bindings: Vec<ModuleItem>,
}

impl RuntimeDelegate {
    pub fn new(id: String, paths: Option<AHashMap<String, String>>) -> Self {
        Self {
            id,
            paths,
            ctx_ident: private_ident!("__ctx"),
            deps: OHashMap::default(),
            exports: Vec::default(),
            hoisted_decls: Vec::default(),
            export_decls: Vec::default(),
            bindings: Vec::default(),
        }
    }
}

impl AstDelegate for RuntimeDelegate {
    fn make_script_body(&mut self, orig_body: Vec<Stmt>) -> Vec<Stmt> {
        let mut new_body = Vec::with_capacity(orig_body.len() + 1);
        new_body.push(global_module_get_ctx_stmt(&self.id, &self.ctx_ident));
        new_body.push(ctx_reset_call(&self.ctx_ident).into_stmt());
        new_body.extend(orig_body);
        new_body
    }

    fn make_module_body(&mut self, mut orig_body: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let exports = mem::take(&mut self.exports);
        let hoisted_decls = mem::take(&mut self.hoisted_decls);
        let bindings = mem::take(&mut self.bindings);

        orig_body.retain(|item| item.is_stmt());

        let deps_items = deps_to_ast(&self.deps, &self.paths);
        let ExportsAst {
            leading_body,
            trailing_body,
        } = exports_to_ast(&self.ctx_ident, exports, crate::phase::ModulePhase::Runtime);
        let total_item_count = deps_items.len()
            + leading_body.len()
            + orig_body.len()
            + bindings.len()
            + trailing_body.len()
            + 1;

        let mut new_body: Vec<ModuleItem> = Vec::with_capacity(total_item_count);
        new_body.push(global_module_get_ctx_stmt(&self.id, &self.ctx_ident).into());
        new_body.push(ctx_reset_call(&self.ctx_ident).into_stmt().into());
        new_body.extend(deps_items);
        new_body.extend(leading_body);
        new_body.extend(hoisted_decls);
        new_body.extend(orig_body);
        new_body.extend(bindings);
        new_body.extend(trailing_body);
        new_body
    }

    fn import(&mut self, import_decl: &mut ImportDecl) {
        let members = to_import_members(&import_decl.specifiers);
        let src = &import_decl.src.value;

        if let Some(mod_ref) = self.deps.get_mut(&src) {
            mod_ref.members.extend(members.into_iter());
        } else {
            self.deps.insert(src, ModuleRef::new(members));
        }
    }

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

        None
    }

    fn export_default_expr(
        &mut self,
        export_default_expr: &mut ExportDefaultExpr,
    ) -> Option<ModuleItem> {
        let orig_expr = export_default_expr.expr.clone();
        let binding_export = BindingExportMember::new("default".into());
        let binding_assign_expr = assign_expr(binding_export.bind_ident.clone(), *orig_expr);

        self.exports.push(ExportRef::Named(NamedExportRef::new(vec![
            ExportMember::Binding(binding_export),
        ])));

        Some(binding_assign_expr.into_stmt().into())
    }

    fn export_named(&mut self, export_named: &mut NamedExport) {
        self.exports
            .push(export_ref_from_named_export(export_named, &self.paths));
    }

    fn export_all(&mut self, export_all: &mut ExportAll) {
        let src = export_all.src.value.clone();
        let id = self
            .paths
            .as_ref()
            .and_then(|paths| paths.get(src.as_str()));

        self.exports
            .push(ExportRef::ReExportAll(ReExportAllRef::new(
                src,
                id.as_ref().map(|id| id.as_str().into()),
                None,
            )));
    }

    fn call_expr(&mut self, call_expr: &mut CallExpr) -> Option<Expr> {
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
                        return Some(require_call(get_src_lit(lit, &self.paths)));
                    }
                    _ => HANDLER.with(|handler| {
                        handler
                            .struct_span_err(callee_expr.span(), "invalid require call")
                            .emit();

                        None
                    }),
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
                        return Some(require_call(get_src_lit(lit, &self.paths)));
                    }
                    _ => HANDLER.with(|handler| {
                        handler
                            .struct_span_err(call_expr.span(), "unsupported dynamic import usage")
                            .emit();

                        None
                    }),
                }
            }
            _ => return None,
        }
    }

    fn assign_expr(&mut self, assign_expr: &mut AssignExpr) -> Option<Expr> {
        to_binding_module_from_assign_expr(
            self.ctx_ident.clone(),
            assign_expr,
            crate::phase::ModulePhase::Runtime,
        )
    }

    fn member_expr(&mut self, member_expr: &mut MemberExpr) -> Option<Expr> {
        to_binding_module_from_member_expr(
            self.ctx_ident.clone(),
            member_expr,
            crate::phase::ModulePhase::Runtime,
        )
    }
}
