use std::mem;

use crate::{
    delegate::{traits::AstDelegate, RegisterDelegate, RuntimeDelegate},
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
    pub deps_id: Option<AHashMap<String, f64>>,
    delegate: Box<dyn AstDelegate>,
}

impl GlobalModuleTransformer {
    /// Returns the AST structure based on the collected module and export data.
    pub fn get_module_body(&mut self, orig_body: Vec<ModuleItem>) -> Vec<ModuleItem> {
        self.delegate.make_body_and_drain(orig_body)
        // let (imports, body, exports) = self.partition_by_module_item(orig_body);

        // // Object properties to be passed to the Global module's export API."
        // let mut export_props: Vec<PropOrSpread> = Vec::new();

        // // An import statement newly added by the re-exports.
        // let mut additional_imports: Vec<ModuleItem> = Vec::new();

        // // A statements that retrieves injected dependencies
        // // through the Global Module's require API.
        // //
        // // ```js
        // // var { ... } = __require('./foo');
        // // var { ... } = __require('./bar');
        // // ```
        // let mut deps_requires: Vec<ModuleItem> = Vec::new();

        // // A list of binding variable declarators,
        // //
        // // ```js
        // // var __x, __x1, __x2;
        // // // => __x, __x1, __x2
        // // ```
        // let mut export_decls: Vec<VarDeclarator> = Vec::new();

        // let ExportsAst {
        //     additional_imports,
        //     deps_requires,
        //     export_decls,
        //     export_props,
        // } = exports_to_ast(mem::take(&mut self.exports), &self.ctx_ident, &self.phase);

        // // TODO:

        // let exports_call = exports_call(&self.ctx_ident, obj_lit_expr(export_props));
        // let exports_decl = VarDecl {
        //     kind: VarDeclKind::Var,
        //     decls: export_decls,
        //     ..Default::default()
        // };

        // let mut new_body = vec![];

        // debug!("additional_imports {:#?}", additional_imports);

        // // Imports
        // if self.phase == ModulePhase::Register {
        //     let (import_stmt, ident) = global_module_import_stmt();
        //     new_body.push(import_stmt);
        //     new_body.extend(imports);
        //     new_body.extend(additional_imports);
        //     new_body.push(
        //         ident
        //             .as_call(DUMMY_SP, vec![num_lit_expr(self.id).as_arg()])
        //             .into_var_decl(VarDeclKind::Var, self.ctx_ident.clone().into())
        //             .into(),
        //     );
        // }

        // // Body
        // if self.phase == ModulePhase::Runtime {
        //     new_body.push(
        //         // `global.__modules.getContext(id);`
        //         member_expr!(Default::default(), DUMMY_SP, global.__modules.getContext)
        //             .as_call(DUMMY_SP, vec![num_lit_expr(self.id).as_arg()])
        //             .into_var_decl(VarDeclKind::Var, self.ctx_ident.clone().into())
        //             .into(),
        //     );
        //     new_body.extend(deps_requires);
        // }
        // let stmts = mem::take(&mut self.stmts);
        // new_body.extend(body);
        // new_body.extend(stmts.into_iter().map(|stmt| stmt.into()));
        // new_body.push(exports_call.into_stmt().into());
        // new_body.push(exports_decl.into());

        // // Exports
        // if self.phase == ModulePhase::Register {
        //     new_body.extend(exports);
        // }

        // new_body
    }
}

impl GlobalModuleTransformer {
    pub fn new(id: f64, phase: ModulePhase, deps_id: Option<AHashMap<String, f64>>) -> Self {
        let delegate: Box<dyn AstDelegate> = match phase {
            ModulePhase::Register => Box::new(RegisterDelegate::new(id)),
            ModulePhase::Runtime => Box::new(RuntimeDelegate::new(id)),
        };

        Self { deps_id, delegate }
    }
}

impl VisitMut for GlobalModuleTransformer {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(self);

        module.body = self.get_module_body(mem::take(&mut module.body));
    }

    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        for item in items.iter_mut() {
            match item {
                // Common statements (It can include require cjs modules or esm dynamic imports)
                //
                // - visit_mut_expr
                //   - call_expr (cjs require, esm dynamic imports)
                //   - assign_expr (TODO: cjs module exports)
                ModuleItem::Stmt(_) => item.visit_mut_children_with(self),
                // Import & Exports (ESModules)
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
            _ => expr.visit_mut_children_with(self),
        }
    }
}
