use core::panic;

use swc_core::{
    atoms::Atom,
    common::{collections::AHashMap, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{private_ident, quote_ident, ExprFactory},
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
};
use tracing::debug;

use crate::{
    constants::{DEPS, EXPORTS, EXPORTS_ARG, REQUIRE_ARG},
    utils::{
        get_expr_from_decl, get_expr_from_default_decl, get_ns_require_call_expr,
        get_require_call_expr, get_require_expr, wrap_with_fn,
    },
};

#[derive(Debug)]
pub enum ModuleRef {
    // `require('...');`
    Require(Require),
    // `import ... from '...';`
    Import(Import),
    // `import('...');`
    DynImport(DynImport),
}

#[derive(Debug)]
pub struct Require {
    pub orig_expr: Expr,
}

impl Require {
    fn new(orig_expr: &Expr) -> Self {
        Require {
            orig_expr: orig_expr.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Import {
    // `import def, { foo, bar as baz } from '...'`;
    // => def, foo, bar (alias: baz)
    pub members: Vec<ModuleMember>,
    // `import * as foo from '...';`
    // => foo
    pub ns: Vec<ModuleMember>,
}

#[derive(Debug)]
pub struct DynImport {
    pub orig_expr: Expr,
}

impl DynImport {
    fn new(orig_expr: &Expr) -> Self {
        DynImport {
            orig_expr: orig_expr.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ModuleMember {
    // `import { foo } from 'foo';`
    // `import * as foo from 'foo';`
    // => foo
    pub ident: Ident,
    // `import { foo as bar } from 'foo'`;
    // => bar
    pub alias: Option<Ident>,
}

#[derive(Debug)]
pub enum ExportRef {
    EsModule(EsModuleExport),
}

#[derive(Debug)]
struct EsModuleExport {
    // Temporary ident for exports.
    exp_ident: Ident,
    // `export { foo }`;
    // `export { x as foo }`;
    // `export { foo } from './foo';`
    // `export * as foo from './foo';`
    // => foo
    exported: Atom,
    // `export * from './foo';`
    // `export * as foo from './foo';`
    re_export: Option<ReExport>,
}

#[derive(Debug)]
enum ReExport {
    Named(NamedReExport),
    Star,
}

#[derive(Debug)]
struct NamedReExport {
    // `export { foo } from './foo';
    // => foo
    ident: Ident,
}

impl ModuleMember {
    fn default(ident: &Ident, alias: Option<Ident>) -> Self {
        ModuleMember {
            ident: ident.clone(),
            alias,
        }
    }
}

pub struct ModuleCollector {
    // `import ... from './foo'`;
    // `require('./foo')`;
    //
    // key: './foo'
    // value: Dep
    pub mods: AHashMap<Atom, ModuleRef>,
    pub exps: Vec<ModuleRef>,
    pub exports_ident: Ident,
    pub require_ident: Ident,
}

impl ModuleCollector {
    pub fn get_deps_ast(&self) -> (Ident, Vec<ModuleItem>, Vec<ModuleItem>) {
        let deps_ident = private_ident!(DEPS);
        let mut dep_props: Vec<PropOrSpread> = Vec::new();
        let mut require_stmts: Vec<ModuleItem> = Vec::new();

        self.mods.iter().for_each(|(key, value)| {
            require_stmts.extend(self.to_require_deps(key, value));
            dep_props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Str(Str {
                    raw: None,
                    span: DUMMY_SP,
                    value: key.clone(),
                }),
                value: Box::new(self.to_dep_obj(value)),
            }))));
        });

        let deps_decl = Expr::Object(ObjectLit {
            props: dep_props,
            ..Default::default()
        })
        .into_var_decl(VarDeclKind::Var, deps_ident.clone().into());

        (deps_ident, vec![deps_decl.into()], require_stmts)
    }

    /// Returns a list of require call expressions that reference modules from global registry.
    ///
    /// ```js
    /// // Examples
    /// function () {
    ///   return {
    ///     foo,
    ///     bar,
    ///     default: baz
    ///   };
    /// }
    /// ```
    fn to_dep_obj(&self, mod_ref: &ModuleRef) -> Expr {
        match mod_ref {
            ModuleRef::Import(Import { members, ns }) => {
                let mut props = members
                    .iter()
                    .map(|imp_member| match &imp_member.alias {
                        Some(alias_ident) => {
                            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                key: imp_member.ident.clone().into(),
                                value: Box::new(alias_ident.clone().into()),
                            })))
                        }
                        None => {
                            PropOrSpread::Prop(Box::new(Prop::Shorthand(imp_member.ident.clone())))
                        }
                    })
                    .collect::<Vec<PropOrSpread>>();

                ns.iter().for_each(|ns| {
                    props.push(PropOrSpread::Spread(SpreadElement {
                        dot3_token: DUMMY_SP,
                        expr: ns.ident.clone().into(),
                    }));
                });

                wrap_with_fn(&Expr::Object(ObjectLit {
                    span: DUMMY_SP,
                    props,
                }))
            }
            ModuleRef::DynImport(dyn_imp) => wrap_with_fn(&dyn_imp.orig_expr),
            ModuleRef::Require(req) => wrap_with_fn(&req.orig_expr),
        }
    }

    /// Returns a list of require call expressions that reference modules from global registry.
    ///
    /// ```js
    /// // Examples
    /// var { default: React, useState, useCallback } = __require('react');
    /// var { core } = __require('@app/core');
    /// ```
    fn to_require_deps(&self, src: &Atom, mod_ref: &ModuleRef) -> Vec<ModuleItem> {
        let mut requires: Vec<ModuleItem> = Vec::new();

        match mod_ref {
            ModuleRef::Import(Import { members, ns }) => {
                ns.iter().for_each(|module| {
                    requires.push(get_ns_require_call_expr(&self.require_ident, src, module));
                });

                if members.len() > 0 {
                    requires.push(get_require_call_expr(&self.require_ident, src, members));
                }
            }
            // Skips AST generation because it has already been replaced during the visit phases.
            ModuleRef::DynImport(_) | ModuleRef::Require(_) => {}
        };

        requires
    }

    fn to_import_members(
        &self,
        specifiers: &Vec<ImportSpecifier>,
    ) -> (Vec<ModuleMember>, Vec<ModuleMember>) {
        let mut members = Vec::with_capacity(specifiers.len());
        let mut ns = Vec::with_capacity(specifiers.len());

        specifiers.iter().for_each(|spec| match spec {
            ImportSpecifier::Default(ImportDefaultSpecifier { local, .. }) => {
                members.push(ModuleMember::default(
                    &quote_ident!("default").into(),
                    Some(local.clone()),
                ));
            }
            ImportSpecifier::Named(ImportNamedSpecifier {
                local,
                imported,
                is_type_only: false,
                ..
            }) => {
                if let Some(ModuleExportName::Ident(ident)) = imported {
                    members.push(ModuleMember::default(ident, Some(local.clone())));
                } else {
                    members.push(ModuleMember::default(local, None));
                }
            }
            ImportSpecifier::Namespace(ImportStarAsSpecifier { local, .. }) => {
                ns.push(ModuleMember::default(local, None));
            }
            _ => {}
        });

        (members, ns)
    }
}

impl Default for ModuleCollector {
    fn default() -> Self {
        ModuleCollector {
            mods: AHashMap::default(),
            exps: Vec::default(),
            exports_ident: private_ident!(EXPORTS_ARG),
            require_ident: private_ident!(REQUIRE_ARG),
        }
    }
}

impl VisitMut for ModuleCollector {
    noop_visit_mut_type!();

    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        for item in items.iter_mut() {
            match item {
                // Statements (It can include CommonJS modules)
                ModuleItem::Stmt(_) => item.visit_mut_children_with(self),
                // Import & Exports (ESModules)
                ModuleItem::ModuleDecl(module_decl) => match module_decl {
                    // Imports
                    //
                    // - `import foo from './foo';`
                    // - `import { foo } from './foo';`
                    // - `import { foo as bar } from './foo';`
                    // - `import * as foo from './foo';`
                    ModuleDecl::Import(import) => {
                        let (members, ns) = self.to_import_members(&import.specifiers);
                        let src = import.src.value.clone();

                        if let Some(ModuleRef::Import(mod_ref)) = self.mods.get_mut(&src) {
                            mod_ref.members.extend(members.into_iter());
                            mod_ref.ns.extend(ns.into_iter());
                        } else {
                            self.mods.insert(
                                import.src.value.clone(),
                                ModuleRef::Import(Import { members, ns }),
                            );
                        }
                    }
                    // Named exports with declarations
                    //
                    // - `export const foo = ...;`
                    // - `export function foo() { ... }`
                    // - `export class Foo { ... }`
                    ModuleDecl::ExportDecl(export_decl) => {
                        let ident = private_ident!(EXPORTS);
                        let assign_expr = AssignExpr {
                            left: AssignTarget::Simple(SimpleAssignTarget::Ident(ident.into())),
                            right: Box::new(get_expr_from_decl(&export_decl.decl)),
                            op: AssignOp::Assign,
                            ..Default::default()
                        };

                        *item = assign_expr.into_stmt().into();

                        // TODO: add exports
                    }
                    // Default exports with declarations
                    //
                    // - `export default function foo() { ... }`
                    // - `export default class Foo { ... }`
                    ModuleDecl::ExportDefaultDecl(export_default_decl) => {
                        let ident = private_ident!(EXPORTS);
                        let assign_expr = AssignExpr {
                            left: AssignTarget::Simple(SimpleAssignTarget::Ident(ident.into())),
                            right: Box::new(get_expr_from_default_decl(&export_default_decl.decl)),
                            op: AssignOp::Assign,
                            ..Default::default()
                        };

                        *item = assign_expr.into_stmt().into();

                        // TODO: add exports
                    }
                    // Named exports
                    //
                    // - `export { foo };`
                    // - `export { foo as bar };`
                    // - `export { foo } from './foo';` (Re-exports)
                    // - `export { foo as bar } from './foo';` (Re-exports)
                    ModuleDecl::ExportNamed(export_named) => {
                        // TODO
                    }
                    // Default exports
                    //
                    // - `export default expr;`
                    ModuleDecl::ExportDefaultExpr(export_default_expr) => {
                        // TODO
                    }
                    // Re-exports specified modules
                    //
                    // - `export * from './foo';`
                    // - `export * as bar from './foo';`
                    ModuleDecl::ExportAll(export_all) => {
                        // TODO
                    }
                    _ => { /* TODO */ }
                },
            }
        }
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        let orig_expr = expr.clone();
        match expr {
            Expr::Call(call_expr) => match call_expr {
                // Collect CommonJS modules.
                //
                // `require('...');`
                CallExpr {
                    args,
                    callee: Callee::Expr(callee_expr),
                    type_args: None,
                    ..
                } if args.len() == 1 && callee_expr.is_ident_ref_to("require") => {
                    let src = args.get(0).unwrap();

                    match &*src.expr {
                        // The first argument of the `require` function must be a string type only.
                        Expr::Lit(Lit::Str(str)) => {
                            self.mods.insert(
                                str.value.clone(),
                                ModuleRef::Require(Require::new(&orig_expr)),
                            );
                            *expr = get_require_expr(&self.require_ident, &str.value, false);
                        }
                        _ => panic!("invalid `require` call expression"),
                    }
                }
                // Collect ESM (Dynamic imports)
                //
                // `import('...', {});`
                CallExpr {
                    args,
                    callee: Callee::Import(_),
                    type_args: None,
                    ..
                } if args.len() >= 1 => {
                    let src = args.get(0).unwrap();

                    match &*src.expr {
                        // The first argument of the `import` function must be a string type only.
                        Expr::Lit(Lit::Str(str)) => {
                            self.mods.insert(
                                str.value.clone(),
                                ModuleRef::DynImport(DynImport::new(&orig_expr)),
                            );
                            *expr = get_require_expr(&self.require_ident, &str.value, false);
                        }
                        _ => panic!("unsupported dynamic import usage"),
                    }
                }
                _ => expr.visit_mut_children_with(self),
            },
            _ => expr.visit_mut_children_with(self),
        }
    }
}
