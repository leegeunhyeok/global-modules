use std::{collections::HashMap, mem};

use swc_core::{
    common::{collections::AHashMap, Spanned, SyntaxContext, DUMMY_SP},
    ecma::{
        ast::*,
        utils::ExprFactory,
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
    plugin::errors::HANDLER,
};
use tracing::debug;

use crate::{
    models::{Dep, Exp},
    utils::ast::{
        assign_expr, export_all_as_exp, export_decl_as_exp, export_default_decl_as_exp,
        export_default_expr_as_exp, export_named_as_exp, get_src_lit, import_as_dep,
        presets::require_call,
    },
};

pub struct ModuleCollector {
    pub deps: Vec<Dep>,
    pub exps: Vec<Exp>,
    unresolved_ctxt: SyntaxContext,
    paths: Option<AHashMap<String, String>>,
}

impl ModuleCollector {
    pub fn new(unresolved_ctxt: SyntaxContext, paths: Option<AHashMap<String, String>>) -> Self {
        Self {
            deps: vec![],
            exps: vec![],
            unresolved_ctxt,
            paths,
        }
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
                    Expr::Lit(lit) => Some(require_call(get_src_lit(lit, &self.paths))),
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
}

impl VisitMut for ModuleCollector {
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
                            if let Some((exp, exp_binding)) = export_decl_as_exp(export_decl) {
                                *item = exp_binding.to_assign_expr().into_stmt().into();
                                self.exps.push(exp);
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
                            if let Some((exp, exp_binding)) =
                                export_default_decl_as_exp(export_default_decl)
                            {
                                *item = exp_binding.to_assign_expr().into_stmt().into();
                                self.exps.push(exp);
                            }
                        }
                        // Default export statements.
                        //
                        // ```js
                        // export default <expr>;
                        // ```
                        ModuleDecl::ExportDefaultExpr(export_default_expr) => {
                            export_default_expr.visit_mut_children_with(self);
                            let (exp, exp_binding) =
                                export_default_expr_as_exp(export_default_expr);
                            *item = exp_binding.to_assign_expr().into_stmt().into();
                            self.exps.push(exp);
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
                                let is_re_export = exp.is_re_export();
                                self.exps.push(exp);

                                if is_re_export {
                                    // Do nothing when it's re-export statement
                                } else {
                                    *item = Expr::from(SeqExpr {
                                        exprs: exp_bindings
                                            .into_iter()
                                            .map(|exp_binding| exp_binding.to_assign_expr().into())
                                            .collect::<Vec<Box<Expr>>>(),
                                        ..Default::default()
                                    })
                                    .into_stmt()
                                    .into();
                                }
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
