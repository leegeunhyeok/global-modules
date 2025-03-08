use std::{collections::HashMap, mem};

use swc_core::{
    common::{collections::AHashMap, util::take::Take, Spanned, SyntaxContext, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{private_ident, ExprFactory},
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
    plugin::errors::HANDLER,
};
use tracing::debug;

use crate::{
    models::{Dep, Exp, ExpBinding},
    utils::ast::{
        assign_expr, export_all_as_exp, export_decl_as_exp, export_default_decl_as_exp,
        export_default_expr_as_exp, export_named_as_exp, get_src_lit, import_as_dep,
        presets::require_call,
    },
};

pub struct ModuleCollector<'a> {
    pub deps: Vec<Dep>,
    pub exps: Vec<Exp>,
    pub exp_bindings: Vec<ExpBinding>,
    unresolved_ctxt: SyntaxContext,
    ctx_ident: &'a Ident,
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

    fn as_require_expr(&mut self, call_expr: &mut CallExpr) -> Option<Expr> {
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
            } if args.len() == 1
                && callee_expr.is_ident_ref_to("require")
                && callee_expr.as_ident().unwrap().ctxt == self.unresolved_ctxt =>
            {
                match &*args[0].expr {
                    // The first argument of the `require` function must be a string type only.
                    Expr::Lit(lit) => {
                        Some(require_call(self.ctx_ident, get_src_lit(lit, &self.paths)))
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
                        return Some(require_call(self.ctx_ident, get_src_lit(lit, &self.paths)));
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
}

impl<'a> VisitMut for ModuleCollector<'a> {
    noop_visit_mut_type!();

    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        debug!("items: {:?}", items);

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
            Expr::Call(call_expr) => {
                if let Some(new_expr) = self.as_require_expr(call_expr) {
                    *expr = new_expr;
                } else {
                    call_expr.visit_mut_children_with(self);
                }
            }
            // TODO
            // Expr::Assign(assign_expr) => {
            //     if let Some(new_expr) = self.delegate.assign_expr(assign_expr) {
            //         *expr = new_expr;
            //     } else {
            //         assign_expr.visit_mut_children_with(self);
            //     }
            // }
            // Expr::Member(member_expr) => {
            //     if let Some(new_expr) = self.delegate.member_expr(member_expr) {
            //         *expr = new_expr;
            //     } else {
            //         member_expr.visit_mut_children_with(self);
            //     }
            // }
            _ => expr.visit_mut_children_with(self),
        }
    }
}
