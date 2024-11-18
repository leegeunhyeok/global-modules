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
    models::{
        DynImportRef, ExportMember, ExportRef, ImportMember, ImportRef, ModuleRef, NamedExportRef,
        NamedReExportRef, ReExportAllRef, RequireRef,
    },
    utils::{
        ast::{
            assign_expr, import_star, kv_prop, lazy_eval_expr, obj_lit_expr, require_call_stmt,
            require_expr, spread_prop, var_declarator,
        },
        parse::{get_expr_from_decl, get_expr_from_default_decl},
    },
};

pub struct ModuleAst {
    /// Additional import statements for re-exports
    ///
    /// ```js
    /// // Before
    /// export * from './foo';
    /// export * as bar from './bar';
    ///
    /// // After
    /// import * as __x from './foo';
    /// import * as __x1 from './bar';
    ///
    /// exports ...;
    /// ```
    pub imp_stmts: Vec<ModuleItem>,
    pub deps_ident: Ident,
    pub deps_decl: Vec<ModuleItem>,
    pub deps_requires: Vec<ModuleItem>,
    pub exps_assigns: Vec<ModuleItem>,
    pub exps_call: Vec<ModuleItem>,
    pub exps_decl: Vec<ModuleItem>,
}

impl ModuleAst {
    pub fn create(
        imp_stmts: Vec<ModuleItem>,
        deps_ident: Ident,
        deps_decl: VarDecl,
        deps_requires: Vec<ModuleItem>,
        exps_assigns: Vec<ModuleItem>,
        exps_call: Expr,
        exps_decl: VarDecl,
    ) -> Self {
        ModuleAst {
            imp_stmts,
            deps_ident,
            deps_decl: vec![deps_decl.into()],
            deps_requires,
            exps_assigns,
            exps_call: vec![exps_call.into_stmt().into()],
            exps_decl: vec![exps_decl.into()],
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
        let mut exps_assigns: Vec<ModuleItem> = Vec::new();
        let mut exps_decls: Vec<VarDeclarator> = Vec::new();

        for key in self.mods_idx.iter() {
            let value = self.mods.get(key).unwrap();

            if let Some(stmts) = self.to_require_deps(key, value) {
                require_stmts.extend(stmts);
            }

            dep_props.push(kv_prop(key, self.to_dep_obj(value)));
        }

        self.exps.iter().for_each(|exp_ref| match exp_ref {
            ExportRef::Named(NamedExportRef { members }) => members.iter().for_each(|member| {
                exps_assigns.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
                    span: DUMMY_SP,
                    expr: member
                        .orig_ident
                        .clone()
                        .make_assign_to(AssignOp::Assign, member.exp_ident.clone().into())
                        .into(),
                })));
                exp_props.push(kv_prop(&member.name, member.exp_ident.clone().into()));
                exps_decls.push(var_declarator(&member.exp_ident));
            }),
            ExportRef::NamedReExport(NamedReExportRef {
                ident,
                src,
                members,
            }) => {
                members.iter().for_each(|member| {
                    exp_props.push(kv_prop(
                        &member.name,
                        ident.clone().make_member(member.name.clone().into()).into(),
                    ));
                    exps_decls.push(var_declarator(&member.exp_ident));
                });

                imp_stmts.push(import_star(ident, src));
            }
            ExportRef::ReExportAll(ReExportAllRef { ident, src, name }) => {
                exp_props.push(match name {
                    Some(exp_name) => kv_prop(exp_name, ident.clone().into()),
                    None => spread_prop(ident.clone().into()),
                });
                exps_decls.push(var_declarator(&ident));
                imp_stmts.push(import_star(ident, src));
            }
        });

        let deps_decl =
            obj_lit_expr(dep_props).into_var_decl(VarDeclKind::Var, deps_ident.clone().into());

        let exps_decl = VarDecl {
            kind: VarDeclKind::Var,
            decls: exps_decls,
            ..Default::default()
        };

        let exps_call = self
            .exports_ident
            .clone()
            .as_call(DUMMY_SP, vec![obj_lit_expr(exp_props).into()]);

        ModuleAst::create(
            imp_stmts,
            deps_ident,
            deps_decl,
            require_stmts,
            exps_assigns,
            exps_call,
            exps_decl,
        )
    }

    fn reg_import(&mut self, src: &Atom, members: Vec<ImportMember>) {
        if let Some(ModuleRef::Import(mod_ref)) = self.mods.get_mut(&src) {
            mod_ref.members.extend(members.into_iter());
        } else {
            self.reg_dep(src, ModuleRef::Import(ImportRef::new(members)));
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
            ModuleRef::Import(ImportRef { members }) => {
                let props = members
                    .iter()
                    .map(|imp_member| match imp_member {
                        // Before
                        //
                        // ```js
                        // export * from './foo';
                        // ```
                        //
                        // After
                        //
                        // ```js
                        // import * as __x from './foo';
                        //
                        // { ...__x };
                        // ```
                        ImportMember {
                            ident, is_ns: true, ..
                        } => spread_prop(ident.clone().into()),
                        // Before
                        //
                        // ```js
                        // export { foo as bar } from './foo';
                        // ```
                        //
                        // After
                        //
                        // ```js
                        // import { foo as __x } from './foo';
                        //
                        // { "bar": __x };
                        // ```
                        ImportMember {
                            ident,
                            alias: Some(alias_ident),
                            ..
                        } => kv_prop(&ident.sym, alias_ident.clone().into()),
                        // Before
                        //
                        // ```js
                        // export { foo } from './foo';
                        // ```
                        //
                        // After
                        //
                        // ```js
                        // { "foo": foo };
                        // ```
                        ImportMember {
                            ident, alias: None, ..
                        } => PropOrSpread::Prop(Box::new(Prop::Shorthand(ident.clone()))),
                    })
                    .collect::<Vec<PropOrSpread>>();

                lazy_eval_expr(&obj_lit_expr(props))
            }
            ModuleRef::DynImport(dyn_imp) => lazy_eval_expr(&dyn_imp.orig_expr),
            ModuleRef::Require(req) => lazy_eval_expr(&req.orig_expr),
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
            ModuleRef::Import(ImportRef { members }) => {
                members
                    .iter()
                    .for_each(|module_member| match module_member {
                        ImportMember { is_ns: true, .. } => requires.push(require_call_stmt(
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
            requires.push(require_call_stmt(
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
                    let name_ident = if let Some(ModuleExportName::Ident(exported_ident)) = exported
                    {
                        exported_ident
                    } else {
                        orig_ident
                    };

                    members.push(ExportMember::new(orig_ident, Some(name_ident.sym.clone())));
                }
                ModuleExportName::Str(_) => unimplemented!("TODO"),
            },
            ExportSpecifier::Namespace(ExportNamespaceSpecifier { name, .. }) => match name {
                ModuleExportName::Ident(orig_ident) => {
                    members.push(ExportMember::new(orig_ident, None))
                }
                ModuleExportName::Str(_) => unimplemented!("TODO"),
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
                // Common statements (It can include require cjs modules or esm dynamic imports)
                //
                // - visit_mut_expr
                //   - call_expr (cjs require, esm dynamic imports)
                //   - assign_expr (TODO: cjs module exports)
                ModuleItem::Stmt(_) => item.visit_mut_children_with(self),
                // Import & Exports (ESModules)
                ModuleItem::ModuleDecl(module_decl) => match module_decl {
                    // Import statements.
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
                    // Named export statements with declarations.
                    //
                    // - `export const foo = ...;`
                    // - `export function foo() { ... }`
                    // - `export class Foo { ... }`
                    ModuleDecl::ExportDecl(export_decl) => {
                        let (orig_ident, decl_expr) = get_expr_from_decl(&export_decl.decl);
                        let member = ExportMember::anonymous(orig_ident.sym);

                        *item = assign_expr(&member.exp_ident, decl_expr).into_stmt().into();

                        self.exps
                            .push(ExportRef::Named(NamedExportRef::new(vec![member])));
                    }
                    // Default export statements with declarations.
                    //
                    // - `export default function foo() { ... }`
                    // - `export default class Foo { ... }`
                    ModuleDecl::ExportDefaultDecl(export_default_decl) => {
                        let member = ExportMember::anonymous(Atom::new("default"));

                        *item = assign_expr(
                            &member.exp_ident,
                            get_expr_from_default_decl(&export_default_decl.decl),
                        )
                        .into_stmt()
                        .into();

                        self.exps
                            .push(ExportRef::Named(NamedExportRef::new(vec![member])));
                    }
                    // Named export statements.
                    //
                    // - `export { foo };`
                    // - `export { foo as bar };`
                    // - `export { foo } from './foo';` (Re-exports)
                    // - `export { foo as bar } from './foo';` (Re-exports)
                    // - `export * as bar from './foo';` (Re-exports)
                    ModuleDecl::ExportNamed(export_named) => {
                        match &export_named.src {
                            // Re-exports
                            Some(src_str) => {
                                let specifier = export_named
                                    .specifiers
                                    .get(0)
                                    .expect("invalid named re-export all");
                                let ident = private_ident!(EXPORTS);
                                let src = src_str.clone().value;

                                self.reg_import(&src, vec![ImportMember::ns(&ident)]);

                                match specifier {
                                    // Re-export without alias name.
                                    //
                                    // ```js
                                    // export * from './foo';
                                    // ```
                                    ExportSpecifier::Namespace(named_exp) => {
                                        self.exps.push(ExportRef::ReExportAll(ReExportAllRef {
                                            ident,
                                            src,
                                            name: Some(named_exp.name.atom().clone()),
                                        }));
                                    }
                                    // Re-export with alias name.
                                    //
                                    // ```js
                                    // export * as foo from './foo';
                                    // ```
                                    _ => {
                                        self.exps.push(ExportRef::NamedReExport(
                                            NamedReExportRef {
                                                ident,
                                                src,
                                                members: self
                                                    .to_export_members(&export_named.specifiers),
                                            },
                                        ));
                                    }
                                }
                            }
                            // Named export
                            None => {
                                let members = self.to_export_members(&export_named.specifiers);
                                self.exps
                                    .push(ExportRef::Named(NamedExportRef::new(members)))
                            }
                        }
                    }
                    // Default export statements.
                    //
                    // - `export default expr;`
                    ModuleDecl::ExportDefaultExpr(ExportDefaultExpr { expr, .. }) => {
                        let orig_expr = *expr.clone();
                        let member = ExportMember::anonymous(Atom::new("default"));

                        *item = assign_expr(&member.exp_ident, orig_expr).into_stmt().into();

                        self.exps
                            .push(ExportRef::Named(NamedExportRef::new(vec![member])));
                    }
                    // Re-exports all statements.
                    //
                    // - `export * from './foo';`
                    ModuleDecl::ExportAll(ExportAll {
                        src,
                        type_only: false,
                        with: None,
                        ..
                    }) => {
                        let ident = private_ident!(EXPORTS);
                        let src = src.clone().value;

                        self.reg_import(&src, vec![ImportMember::ns(&ident)]);
                        self.exps.push(ExportRef::ReExportAll(ReExportAllRef {
                            src,
                            ident: ident.clone(),
                            name: None,
                        }));
                    }
                    _ => {}
                },
            }
        }

        debug!("{:#?}", self.exps);
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        let orig_expr = expr.clone();
        match expr {
            Expr::Call(call_expr) => match call_expr {
                // Collect CommonJS requires.
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
                        Expr::Lit(Lit::Str(str)) => {
                            self.mods.insert(
                                str.value.clone(),
                                ModuleRef::Require(RequireRef::new(&orig_expr)),
                            );
                            *expr = require_expr(&self.require_ident, &str.value, false);
                        }
                        _ => panic!("invalid `require` call expression"),
                    }
                }
                // Collect ESM (Dynamic imports)
                //
                // ```js
                // import('...', {});
                // ```
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
                                ModuleRef::DynImport(DynImportRef::new(&orig_expr)),
                            );
                            *expr = require_expr(&self.require_ident, &str.value, false);
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
