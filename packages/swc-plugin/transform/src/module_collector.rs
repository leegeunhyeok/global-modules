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
        get_assign_expr, get_expr_from_decl, get_expr_from_default_decl, get_require_call_stmt,
        get_require_expr, wrap_with_fn,
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
    pub members: Vec<ImportMember>,
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
pub struct ImportMember {
    // `import { foo } from 'foo';`
    // `import * as foo from 'foo';`
    // => foo
    pub ident: Ident,
    // `import { foo as bar } from 'foo'`;
    // => bar
    pub alias: Option<Ident>,
    // `true` if name spaced import.
    pub is_ns: bool,
}

#[derive(Debug)]
pub enum ExportRef {
    Named(NamedExportRef),
    ReExport(ReExport),
}

#[derive(Debug)]
struct NamedExportRef {
    pub members: Vec<ExportMember>,
}

#[derive(Debug)]
struct ExportMember {
    pub ident: Ident,
    // `export { foo as x };`
    // => foo
    pub name: Atom,
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
    src: Atom,
}

#[derive(Debug)]
struct StarReExport {
    // `export * as foo from './foo';
    // => foo
    ident: Ident,
}

impl ImportMember {
    fn default(ident: &Ident, alias: Option<Ident>) -> Self {
        ImportMember {
            ident: ident.clone(),
            alias,
            is_ns: false,
        }
    }

    fn ns(ident: &Ident) -> Self {
        ImportMember {
            ident: ident.clone(),
            alias: None,
            is_ns: true,
        }
    }
}

pub struct ModuleAst {
    pub imp_stmts: Vec<ModuleItem>,
    pub deps_ident: Ident,
    pub deps_decl: Vec<ModuleItem>,
    pub deps_requires: Vec<ModuleItem>,
    pub exps_call: Vec<ModuleItem>,
}

impl ModuleAst {
    pub fn create(
        imp_stmts: Vec<ModuleItem>,
        deps_ident: Ident,
        deps_decl: VarDecl,
        deps_requires: Vec<ModuleItem>,
        exps_call: Expr,
    ) -> Self {
        ModuleAst {
            imp_stmts,
            deps_ident,
            deps_decl: vec![deps_decl.into()],
            deps_requires,
            exps_call: vec![exps_call.into_stmt().into()],
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
    pub mods_idx: Vec<Atom>,
    pub exps: Vec<ExportRef>,
    pub exports_ident: Ident,
    pub require_ident: Ident,
}

impl ModuleCollector {
    pub fn get_module_ast(&self) -> ModuleAst {
        let deps_ident = private_ident!(DEPS);
        let mut dep_props: Vec<PropOrSpread> = Vec::new();
        let mut exp_props: Vec<PropOrSpread> = Vec::new();
        let mut imp_stmts: Vec<ModuleItem> = Vec::new();
        let mut require_stmts: Vec<ModuleItem> = Vec::new();

        for key in self.mods_idx.iter() {
            let value = self.mods.get(key).unwrap();

            if let Some(stmts) = self.to_require_deps(key, value) {
                require_stmts.extend(stmts);
            }

            dep_props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Str(Str {
                    raw: None,
                    span: DUMMY_SP,
                    value: key.clone(),
                }),
                value: Box::new(self.to_dep_obj(value)),
            }))));
        }

        self.exps.iter().for_each(|exp_ref| match exp_ref {
            ExportRef::Named(named_exp) => exp_props.extend(
                named_exp
                    .members
                    .iter()
                    .map(|member| {
                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Str(member.name.clone().into()),
                            value: member.ident.clone().into(),
                        })))
                    })
                    .collect::<Vec<PropOrSpread>>(),
            ),
            ExportRef::ReExport(re_exp) => match re_exp {
                ReExport::Named(NamedReExport { ident, src }) => {
                    imp_stmts.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                        phase: ImportPhase::Evaluation,
                        specifiers: vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
                            local: ident.clone(),
                            span: DUMMY_SP,
                        })],
                        src: Box::new(src.clone().into()),
                        type_only: false,
                        with: None,
                        span: DUMMY_SP,
                    })));
                }
                ReExport::Star => {
                    // TODO
                }
            },
        });

        let deps_decl = Expr::Object(ObjectLit {
            props: dep_props,
            ..Default::default()
        })
        .into_var_decl(VarDeclKind::Var, deps_ident.clone().into());

        let exps_call = self.exports_ident.clone().as_call(
            DUMMY_SP,
            vec![ExprOrSpread {
                expr: Box::new(Expr::Object(ObjectLit {
                    props: exp_props,
                    ..Default::default()
                })),
                spread: None,
            }],
        );

        ModuleAst::create(imp_stmts, deps_ident, deps_decl, require_stmts, exps_call)
    }

    fn reg_import(&mut self, src: &Atom, members: Vec<ImportMember>) {
        if let Some(ModuleRef::Import(mod_ref)) = self.mods.get_mut(&src) {
            mod_ref.members.extend(members.into_iter());
        } else {
            self.reg_dep(src, ModuleRef::Import(Import { members }));
        }
    }

    fn reg_dep(&mut self, src: &Atom, mod_ref: ModuleRef) {
        self.mods.insert(src.clone(), mod_ref);
        self.mods_idx.push(src.clone());
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
            ModuleRef::Import(Import { members }) => {
                let props = members
                    .iter()
                    .map(|imp_member| match imp_member {
                        ImportMember {
                            ident, is_ns: true, ..
                        } => PropOrSpread::Spread(SpreadElement {
                            dot3_token: DUMMY_SP,
                            expr: ident.clone().into(),
                        }),
                        ImportMember {
                            ident,
                            alias: Some(alias_ident),
                            ..
                        } => PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: ident.clone().into(),
                            value: Box::new(alias_ident.clone().into()),
                        }))),
                        ImportMember {
                            ident, alias: None, ..
                        } => PropOrSpread::Prop(Box::new(Prop::Shorthand(ident.clone()))),
                    })
                    .collect::<Vec<PropOrSpread>>();

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
    fn to_require_deps(&self, src: &Atom, mod_ref: &ModuleRef) -> Option<Vec<ModuleItem>> {
        let mut requires: Vec<ModuleItem> = Vec::new();
        let mut dep_props: Vec<ObjectPatProp> = Vec::new();

        match mod_ref {
            ModuleRef::Import(Import { members }) => {
                members
                    .iter()
                    .for_each(|module_member| match module_member {
                        ImportMember { is_ns: true, .. } => requires.push(get_require_call_stmt(
                            &self.require_ident,
                            src,
                            module_member.ident.clone().into(),
                            true,
                        )),
                        ImportMember {
                            ident,
                            alias: Some(alias_ident),
                            ..
                        } => dep_props.push(ObjectPatProp::KeyValue(KeyValuePatProp {
                            key: PropName::Ident(ident.clone().into()),
                            value: Box::new(Pat::Ident(alias_ident.clone().into())),
                        })),
                        ImportMember {
                            ident, alias: None, ..
                        } => dep_props.push(ObjectPatProp::Assign(AssignPatProp {
                            key: ident.clone().into(),
                            value: None,
                            span: DUMMY_SP,
                        })),
                    });
            }
            // Skips AST generation because it has already been replaced during the visit phases.
            ModuleRef::DynImport(_) | ModuleRef::Require(_) => return None,
        };

        if dep_props.len() > 0 {
            requires.push(get_require_call_stmt(
                &self.require_ident,
                src,
                ObjectPat {
                    props: dep_props,
                    optional: false,
                    type_ann: None,
                    span: DUMMY_SP,
                }
                .into(),
                false,
            ));
        }

        requires.into()
    }

    fn to_import_members(&self, specifiers: &Vec<ImportSpecifier>) -> Vec<ImportMember> {
        let mut members = Vec::with_capacity(specifiers.len());

        specifiers.iter().for_each(|spec| match spec {
            ImportSpecifier::Default(ImportDefaultSpecifier { local, .. }) => {
                members.push(ImportMember::default(
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
                    members.push(ImportMember::default(ident, Some(local.clone())));
                } else {
                    members.push(ImportMember::default(local, None));
                }
            }
            ImportSpecifier::Namespace(ImportStarAsSpecifier { local, .. }) => {
                members.push(ImportMember::ns(local));
            }
            _ => {}
        });

        members
    }

    fn to_export_members(&self, specifiers: &Vec<ExportSpecifier>) -> Vec<ExportMember> {
        let mut members = Vec::with_capacity(specifiers.len());

        specifiers.iter().for_each(|spec| match spec {
            ExportSpecifier::Named(ExportNamedSpecifier {
                orig,
                exported,
                is_type_only: false,
                ..
            }) => match orig {
                ModuleExportName::Ident(orig_ident) => {
                    if let Some(ModuleExportName::Ident(ident)) = exported {
                        members.push(ExportMember {
                            ident: orig_ident.clone(),
                            name: ident.sym.clone(),
                        });
                    } else {
                        members.push(ExportMember {
                            ident: orig_ident.clone(),
                            name: orig_ident.sym.clone(),
                        });
                    }
                }
                ModuleExportName::Str(_) => unimplemented!("todo"),
            },
            ExportSpecifier::Namespace(ExportNamespaceSpecifier { name, .. }) => match name {
                ModuleExportName::Ident(orig_ident) => {
                    members.push(ExportMember {
                        ident: orig_ident.clone(),
                        name: orig_ident.sym.clone(),
                    });
                }
                ModuleExportName::Str(_) => unimplemented!("todo"),
            },
            _ => {}
        });

        members
    }
}

impl Default for ModuleCollector {
    fn default() -> Self {
        ModuleCollector {
            mods: AHashMap::default(),
            mods_idx: Vec::default(),
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
                        let members = self.to_import_members(&import.specifiers);
                        let src = import.src.value.clone();

                        self.reg_import(&src, members);
                    }
                    // Named exports with declarations
                    //
                    // - `export const foo = ...;`
                    // - `export function foo() { ... }`
                    // - `export class Foo { ... }`
                    ModuleDecl::ExportDecl(export_decl) => {
                        let ident = private_ident!(EXPORTS);
                        let (orig_ident, decl_expr) = get_expr_from_decl(&export_decl.decl);

                        self.exps.push(ExportRef::Named(NamedExportRef {
                            members: vec![ExportMember {
                                ident: ident.clone(),
                                name: orig_ident.sym,
                            }],
                        }));

                        *item = get_assign_expr(ident, decl_expr).into_stmt().into();
                    }
                    // Default exports with declarations
                    //
                    // - `export default function foo() { ... }`
                    // - `export default class Foo { ... }`
                    ModuleDecl::ExportDefaultDecl(export_default_decl) => {
                        let ident = private_ident!(EXPORTS);

                        self.exps.push(ExportRef::Named(NamedExportRef {
                            members: vec![ExportMember {
                                ident: ident.clone(),
                                name: Atom::new("default"),
                            }],
                        }));

                        *item = get_assign_expr(
                            ident,
                            get_expr_from_default_decl(&export_default_decl.decl),
                        )
                        .into_stmt()
                        .into();
                    }
                    // Named exports
                    //
                    // - `export { foo };`
                    // - `export { foo as bar };`
                    // - `export { foo } from './foo';` (Re-exports)
                    // - `export { foo as bar } from './foo';` (Re-exports)
                    // - `export * as bar from './foo';` (Re-exports)
                    ModuleDecl::ExportNamed(export_named) => {
                        match &export_named.src {
                            // Re-export
                            Some(src_str) => {
                                let ident = private_ident!(EXPORTS);
                                let src = src_str.clone().value;

                                self.reg_import(&src, vec![ImportMember::ns(&ident)]);
                                self.exps.push(ExportRef::ReExport(ReExport::Named(
                                    NamedReExport { ident, src },
                                )))
                            }
                            // Named export
                            None => {
                                let members: Vec<ExportMember> =
                                    self.to_export_members(&export_named.specifiers);
                                let ident = private_ident!(EXPORTS);
                                self.exps.push(ExportRef::Named(NamedExportRef { members }));

                                // TODO
                            }
                        }
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
                    ModuleDecl::ExportAll(export_all) => {
                        // TODO
                    }
                    _ => {}
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
