use std::mem;

use crate::{
    delegate::{traits::AstDelegate, BundleDelegate, RuntimeDelegate},
    phase::ModulePhase,
};
use swc_core::{
    common::collections::AHashMap,
    ecma::{
        ast::*,
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
};

pub struct GlobalModuleTransformer {
    delegate: Box<dyn AstDelegate>,
}

impl GlobalModuleTransformer {
    pub fn get_script_body(&mut self, orig_body: Vec<Stmt>) -> Vec<Stmt> {
        self.delegate.make_script_body(orig_body)
    }

    pub fn get_module_body(&mut self, orig_body: Vec<ModuleItem>) -> Vec<ModuleItem> {
        self.delegate.make_module_body(orig_body)
    }
}

impl GlobalModuleTransformer {
    pub fn new(id: f64, phase: ModulePhase, deps_id: Option<AHashMap<String, f64>>) -> Self {
        let delegate: Box<dyn AstDelegate> = match phase {
            ModulePhase::Bundle => Box::new(BundleDelegate::new(id)),
            ModulePhase::Runtime => Box::new(RuntimeDelegate::new(id, deps_id)),
        };

        Self { delegate }
    }
}

impl VisitMut for GlobalModuleTransformer {
    noop_visit_mut_type!();

    fn visit_mut_script(&mut self, script: &mut Script) {
        script.visit_mut_children_with(self);

        // Replace to new script body.
        script.body = self.get_script_body(mem::take(&mut script.body));
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(self);

        // Replace to new module body.
        module.body = self.get_module_body(mem::take(&mut module.body));
    }

    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        for item in items.iter_mut() {
            match item {
                // Statements (It can include CommonJS's require call and module exports / ESModule's dynamic imports)
                ModuleItem::Stmt(_) => item.visit_mut_children_with(self),
                // Imports & Exports (ESModule)
                ModuleItem::ModuleDecl(module_decl) => {
                    match module_decl {
                        // Import statements.
                        //
                        // ```js
                        // import foo from './foo';
                        // import { foo } from './foo';
                        // import { foo as bar } from './foo';
                        // import * as foo from './foo';
                        // ```
                        ModuleDecl::Import(import_decl) => self.delegate.import(import_decl),
                        // Named export statements with declarations.
                        //
                        // ```js
                        // export const foo = ...;
                        // export function foo() { ... }
                        // export class Foo { ... }
                        // ```
                        ModuleDecl::ExportDecl(export_decl) => {
                            export_decl.visit_mut_children_with(self);

                            *item = self.delegate.export_decl(&export_decl);
                        }
                        // Default export statements with declarations.
                        //
                        // ```js
                        // export default function foo() { ... }
                        // export default class Foo { ... }
                        // ```
                        ModuleDecl::ExportDefaultDecl(export_default_decl) => {
                            export_default_decl.visit_mut_children_with(self);

                            *item = self.delegate.export_default_decl(export_default_decl)
                        }
                        // Default export statements.
                        //
                        // ```js
                        // export default <expr>;
                        // ```
                        ModuleDecl::ExportDefaultExpr(export_default_expr) => {
                            export_default_expr.visit_mut_children_with(self);

                            if let Some(new_item) =
                                self.delegate.export_default_expr(export_default_expr)
                            {
                                export_default_expr.expr = new_item.into()
                            }
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
                        ) => self.delegate.export_named(export_named),
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
                        ) => self.delegate.export_all(export_all),
                        _ => {}
                    }
                }
            }
        }
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Call(call_expr) => {
                if let Some(new_expr) = self.delegate.call_expr(call_expr) {
                    *expr = new_expr;
                } else {
                    call_expr.visit_mut_children_with(self);
                }
            }
            Expr::Assign(assign_expr) => {
                if let Some(new_expr) = self.delegate.assign_expr(assign_expr) {
                    *expr = new_expr;
                } else {
                    assign_expr.visit_mut_children_with(self);
                }
            }
            Expr::Member(member_expr) => {
                if let Some(new_expr) = self.delegate.member_expr(member_expr) {
                    *expr = new_expr;
                } else {
                    member_expr.visit_mut_children_with(self);
                }
            }
            _ => expr.visit_mut_children_with(self),
        }
    }
}
