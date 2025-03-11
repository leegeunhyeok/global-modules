use std::mem;

use swc_core::{
    common::{collections::AHashMap, util::take::Take, Spanned, SyntaxContext},
    ecma::{
        ast::*,
        utils::ExprFactory,
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
    plugin::errors::HANDLER,
};
use tracing::debug;

use crate::{
    models::{Dep, Exp, ExpBinding},
    utils::ast::{
        export_all_as_exp, export_decl_as_exp, export_default_decl_as_exp,
        export_default_expr_as_exp, export_named_as_exp, get_src, import_as_dep,
        is_cjs_exports_member, is_cjs_module_member, is_require_call,
        presets::{assign_cjs_module_expr, module_exports_member, require_call},
        to_cjs_export_name,
    },
};

pub struct ModuleCollector<'a> {
    pub deps: Vec<Dep>,
    pub exps: Vec<Exp>,
    pub exp_bindings: Vec<ExpBinding>,
    pub ctx_ident: &'a Ident,
    unresolved_ctxt: SyntaxContext,
    paths: &'a Option<AHashMap<String, String>>,
}

impl<'a> ModuleCollector<'a> {
    pub fn new(
        unresolved_ctxt: SyntaxContext,
        ctx_ident: &'a Ident,
        paths: &'a Option<AHashMap<String, String>>,
    ) -> Self {
        Self {
            deps: vec![],
            exps: vec![],
            exp_bindings: vec![],
            ctx_ident,
            unresolved_ctxt,
            paths,
        }
    }

    pub fn take_deps(&mut self) -> Vec<Dep> {
        mem::take(&mut self.deps)
    }

    pub fn take_exps(&mut self) -> Vec<Exp> {
        mem::take(&mut self.exps)
    }

    pub fn take_bindings(&mut self) -> Vec<ExpBinding> {
        mem::take(&mut self.exp_bindings)
    }
}

impl<'a> VisitMut for ModuleCollector<'a> {
    noop_visit_mut_type!();

    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        for item in items.iter_mut() {
            match item {
                // Statements (It can include CommonJS's require call and module exports / ESModule's dynamic imports)
                ModuleItem::Stmt(_) => item.visit_mut_children_with(self),
                // Imports & Exports (ESModule)
                ModuleItem::ModuleDecl(module_decl) => {
                    debug!("module_decl: {:?}", module_decl);

                    match module_decl {
                        // Import statements.
                        //
                        // ```js
                        // import foo from './foo';
                        // import { foo } from './foo';
                        // import { foo as bar } from './foo';
                        // import * as foo from './foo';
                        // ```
                        ModuleDecl::Import(import_decl) => {
                            if let Some(dep) = import_as_dep(import_decl) {
                                self.deps.push(dep);
                            }
                        }
                        // Named export statements with declarations.
                        //
                        // ```js
                        // export const foo = ...;
                        // export function foo() { ... }
                        // export class Foo { ... }
                        // ```
                        ModuleDecl::ExportDecl(export_decl) => {
                            export_decl.visit_mut_children_with(self);
                            if let Some((exp, decl_stmt, exp_binding)) =
                                export_decl_as_exp(export_decl)
                            {
                                *item = decl_stmt.into();
                                self.exps.push(exp);
                                self.exp_bindings.push(exp_binding);
                            }
                        }
                        // Default export statements with declarations.
                        //
                        // ```js
                        // export default function foo() { ... }
                        // export default class Foo { ... }
                        // ```
                        ModuleDecl::ExportDefaultDecl(export_default_decl) => {
                            export_default_decl.visit_mut_children_with(self);
                            if let Some((exp, decl, exp_binding)) =
                                export_default_decl_as_exp(export_default_decl)
                            {
                                *item = decl.into();
                                self.exps.push(exp);
                                self.exp_bindings.push(exp_binding)
                            }
                        }
                        // Default export statements.
                        //
                        // ```js
                        // export default <expr>;
                        // ```
                        ModuleDecl::ExportDefaultExpr(export_default_expr) => {
                            export_default_expr.visit_mut_children_with(self);
                            let (exp, stmt, exp_binding) =
                                export_default_expr_as_exp(export_default_expr);
                            *item = stmt.into();
                            self.exps.push(exp);
                            self.exp_bindings.push(exp_binding);
                        }
                        // Named export statements.
                        //
                        // ```js
                        // export { foo };
                        // export { foo as bar };
                        //
                        // // Named re-exports
                        // export { foo } from './foo';
                        // export { foo as bar } from './foo';
                        // export { default } from './foo';
                        // export { default as foo } from './foo';
                        // export * as bar from './foo';
                        // ```
                        ModuleDecl::ExportNamed(
                            export_named @ NamedExport {
                                type_only: false,
                                with: None,
                                ..
                            },
                        ) => {
                            if let Some((exp, exp_bindings)) = export_named_as_exp(export_named) {
                                match exp {
                                    Exp::Default(_) => {
                                        self.exp_bindings.extend(exp_bindings);
                                        item.take();
                                    }
                                    _ => {}
                                }

                                self.exps.push(exp);
                            }
                        }
                        // Re-exports all statements.
                        //
                        // ```js
                        // export * from './foo';
                        // ```
                        ModuleDecl::ExportAll(
                            export_all @ ExportAll {
                                type_only: false,
                                with: None,
                                ..
                            },
                        ) => self.exps.push(export_all_as_exp(export_all)),
                        _ => {}
                    }
                }
            }
        }
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            // CommonJS's require call
            Expr::Call(
                call_expr @ CallExpr {
                    callee: Callee::Expr(_),
                    type_args: None,
                    ..
                },
            ) if is_require_call(self.unresolved_ctxt, call_expr) => {
                match &*call_expr.args[0].expr {
                    // The first argument of the `require` function must be a string type only.
                    Expr::Lit(lit) => {
                        let src = get_src(lit, &self.paths);
                        self.deps.push(Dep::lazy(src.clone(), expr.clone()));
                        *expr = require_call(self.ctx_ident, Lit::Str(src.into()));
                    }
                    _ => HANDLER.with(|handler| {
                        handler
                            .struct_span_err(call_expr.span(), "invalid require call")
                            .emit();
                    }),
                }
            }
            // ESModule's dynamic import call
            Expr::Call(
                call_expr @ CallExpr {
                    callee: Callee::Import(_),
                    type_args: None,
                    ..
                },
            ) => {
                let maybe_src = call_expr.args.get(0).expect("invalid dynamic import call");

                match &*maybe_src.expr {
                    // The first argument of the `import` function must be a string type only.
                    Expr::Lit(lit) => {
                        let src = get_src(lit, &self.paths);
                        self.deps.push(Dep::lazy(src.clone(), expr.clone()));
                        *expr = require_call(self.ctx_ident, Lit::Str(src.into()));
                    }
                    _ => HANDLER.with(|handler| {
                        handler
                            .struct_span_err(call_expr.span(), "unsupported dynamic import usage")
                            .emit();
                    }),
                }
            }
            Expr::Assign(
                assign_expr @ AssignExpr {
                    op: AssignOp::Assign,
                    ..
                },
            ) => match &assign_expr.left {
                AssignTarget::Simple(SimpleAssignTarget::Member(member_expr)) => {
                    let module_assign_expr =
                        if is_cjs_exports_member(self.unresolved_ctxt, member_expr) {
                            Some(assign_cjs_module_expr(
                                self.ctx_ident,
                                *assign_expr.right.clone(),
                                to_cjs_export_name(&member_expr.prop).into(),
                            ))
                        } else if is_cjs_module_member(self.unresolved_ctxt, member_expr) {
                            Some(assign_cjs_module_expr(
                                self.ctx_ident,
                                *assign_expr.right.clone(),
                                None,
                            ))
                        } else if let Some(leading_member) = member_expr.obj.as_member() {
                            // `module.exports.foo = ...;`
                            // `module.exports['foo'] = ...;`
                            if is_cjs_module_member(self.unresolved_ctxt, leading_member) {
                                Some(assign_cjs_module_expr(
                                    self.ctx_ident,
                                    *assign_expr.right.clone(),
                                    to_cjs_export_name(&member_expr.prop).into(),
                                ))
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                    if let Some(module_assign_expr) = module_assign_expr {
                        assign_expr.right = Box::new(module_assign_expr);
                    } else {
                        expr.visit_mut_children_with(self);
                    }
                }
                _ => expr.visit_mut_children_with(self),
            },
            Expr::Member(member_expr)
                if is_cjs_module_member(self.unresolved_ctxt, member_expr) =>
            {
                *expr = module_exports_member(self.ctx_ident)
                    .make_assign_to(AssignOp::Assign, member_expr.clone().into());
            }
            _ => expr.visit_mut_children_with(self),
        }
    }
}

pub fn create_collector<'a>(
    unresolved_ctxt: SyntaxContext,
    ctx_ident: &'a Ident,
    paths: &'a Option<AHashMap<String, String>>,
) -> ModuleCollector<'a> {
    ModuleCollector::new(unresolved_ctxt, ctx_ident, paths)
}
