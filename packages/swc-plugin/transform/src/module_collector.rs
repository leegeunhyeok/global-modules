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

use crate::constants::*;
use crate::models::*;
use crate::utils::{ast::*, parse::*};

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
    pub deps_requires: Vec<Stmt>,
    pub exps_assigns: Vec<Stmt>,
    pub exps_call: Vec<Stmt>,
    pub exps_decl: Vec<ModuleItem>,
}

impl ModuleAst {
    pub fn create(
        imp_stmts: Vec<ModuleItem>,
        deps_ident: Ident,
        deps_decl: VarDecl,
        deps_requires: Vec<Stmt>,
        exps_assigns: Vec<Stmt>,
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

pub struct ModuleCollector<'a> {
    exports_ident: &'a Ident,
    require_ident: &'a Ident,
    // `import ... from './foo'`;
    // `require('./foo')`;
    //
    // key: './foo'
    // value: Dep
    mods: AHashMap<Atom, ModuleRef>,
    mods_idx: Vec<Atom>,
    exps: Vec<ExportRef>,
}

impl<'a> ModuleCollector<'a> {
    /// Returns the AST structure based on the collected module and export data.
    pub fn get_module_ast(&self) -> ModuleAst {
        // Ident for declare dependencies.
        //
        // ```js
        // var __deps = { ... };
        // // => __deps
        // ```
        let deps_ident = private_ident!(DEPS);

        // Properties to inject the dependency.
        // (key: module source, value: a function that returns the actual module reference)
        //
        // ```js
        // var __deps = {
        //   './foo': function () {
        //     return <actualFoo>;
        //   },
        // };
        // ```
        let mut dep_props: Vec<PropOrSpread> = Vec::new();

        // Object properties to be passed to the Global module's export API."
        let mut exp_props: Vec<PropOrSpread> = Vec::new();

        // An import statement newly added by the re-exports.
        let mut imp_stmts: Vec<ModuleItem> = Vec::new();

        // A statements that retrieves injected dependencies
        // through the Global Module's require API.
        //
        // ```js
        // var { ... } = __require('./foo');
        // var { ... } = __require('./bar');
        // ```
        let mut require_stmts: Vec<Stmt> = Vec::new();

        // An expressions that assigns the module reference value to the export variable.
        //
        // ```js
        // __x = foo;
        // __x1 = bar;
        // __x2 = bar;
        // ```
        let mut exps_assigns: Vec<Stmt> = Vec::new();

        // A list of export variable declarators,
        //
        // ```js
        // var __x, __x1, __x2;
        // // => __x, __x1, __x2
        // ```
        let mut exps_decls: Vec<VarDeclarator> = Vec::new();

        dep_props.extend(self.mods_idx.iter().map(|src| {
            let value = self.mods.get(src).unwrap();
            if let Some(stmts) = self.to_require_dep_stmts(src, value) {
                require_stmts.extend(stmts);
            }
            kv_prop(src, self.to_dep_obj(value))
        }));

        self.exps.iter().for_each(|exp_ref| match exp_ref {
            ExportRef::Named(NamedExportRef { members }) => members.iter().for_each(|member| {
                if let Some(orig_ident) = &member.orig_ident {
                    exps_assigns.push(
                        orig_ident
                            .clone()
                            .make_assign_to(AssignOp::Assign, member.export_ident.clone().into())
                            .into_stmt(),
                    );
                }

                exp_props.push(kv_prop(&member.name, member.export_ident.clone().into()));
                exps_decls.push(var_declarator(&member.export_ident));
            }),
            ExportRef::NamedReExport(NamedReExportRef {
                mod_ident,
                exp_ident,
                src,
                members,
            }) => {
                imp_stmts.push(import_star(mod_ident, src));
                exp_props.extend(members.iter().map(|member| {
                    kv_prop(
                        &member.name,
                        exp_ident
                            .clone()
                            .make_member(member.name.clone().into())
                            .into(),
                    )
                }));
                exps_decls.push(var_declarator(&exp_ident));
            }
            ExportRef::ReExportAll(ReExportAllRef {
                mod_ident,
                exp_ident,
                src,
                name,
            }) => {
                exp_props.push(match name {
                    Some(exp_name) => kv_prop(exp_name, exp_ident.clone().into()),
                    None => spread_prop(self.to_ns_export(exp_ident.clone().into())),
                });
                exps_decls.push(var_declarator(&exp_ident));
                imp_stmts.push(import_star(mod_ident, src));
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

    /// Register module reference by members.
    fn register_mod_by_members(&mut self, src: &Atom, members: Vec<ImportMember>) {
        if let Some(ModuleRef::Import(mod_ref)) = self.mods.get_mut(&src) {
            mod_ref.members.extend(members.into_iter());
        } else {
            let new_import = ModuleRef::Import(ImportRef::new(members));
            self.insert_mod(src, new_import);
        }
    }

    /// Register module reference if not registered yet.
    fn register_mod(&mut self, src: &Atom, module_ref: ModuleRef) {
        if self.mods.get_mut(src).is_none() {
            self.insert_mod(src, module_ref);
        }
    }

    /// Register export reference.
    fn reg_export(&mut self, export: ExportRef) {
        self.exps.push(export);
    }

    /// Inserts the module reference while ensuring the insertion order.
    fn insert_mod(&mut self, src: &Atom, module_ref: ModuleRef) {
        self.mods.insert(src.clone(), module_ref);
        self.mods_idx.push(src.clone());
    }

    /// Convert `ImportSpecifier` into `ImportMember`.
    fn to_import_members(&self, specifiers: &Vec<ImportSpecifier>) -> Vec<ImportMember> {
        let mut members = Vec::with_capacity(specifiers.len());

        specifiers.iter().for_each(|spec| match spec {
            ImportSpecifier::Default(ImportDefaultSpecifier { local, .. }) => {
                members.push(ImportMember::Named(ImportNamedMember::new(
                    &quote_ident!("default").into(),
                    Some(local.clone()),
                )));
            }
            ImportSpecifier::Named(ImportNamedSpecifier {
                local,
                imported,
                is_type_only: false,
                ..
            }) => {
                if let Some(ModuleExportName::Ident(ident)) = imported {
                    members.push(ImportMember::Named(ImportNamedMember::new(
                        ident,
                        Some(local.clone()),
                    )));
                } else {
                    members.push(ImportMember::Named(ImportNamedMember::new(local, None)));
                }
            }
            ImportSpecifier::Namespace(ImportStarAsSpecifier { local, .. }) => {
                members.push(ImportMember::Namespace(ImportNamespaceMember::alias(local)));
            }
            _ => {}
        });

        members
    }

    /// Convert `ExportSpecifier` into `ExportMember`.
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
                    let export_name =
                        if let Some(ModuleExportName::Ident(exported_ident)) = exported {
                            exported_ident
                        } else {
                            orig_ident
                        }
                        .sym
                        .clone();

                    members.push(ExportMember::new(orig_ident, Some(export_name)));
                }
                ModuleExportName::Str(_) => unimplemented!("TODO"),
            },
            _ => {}
        });

        members
    }

    /* ----- AST Helpers ----- */

    /// Returns global module's require call expression.
    ///
    /// ```js
    /// __require(src);
    /// ```
    fn require_call(&self, src: &Atom) -> Expr {
        self.require_ident
            .clone()
            .as_call(DUMMY_SP, vec![src.clone().as_arg()])
    }

    /// ```js
    /// var { foo, bar, default: baz } = __require('./foo');
    /// ```
    fn decl_required_deps_stmt(&self, src: &Atom, pat: Pat) -> Stmt {
        self.require_call(src)
            .into_var_decl(VarDeclKind::Var, pat)
            .into()
    }

    /// Wrap expression with function for lazy evaluation.
    ///
    /// ```js
    /// function () {
    ///   return /* expr */;
    /// }
    /// ```
    fn to_lazy(&self, expr: &Expr) -> Expr {
        Function {
            body: Some(BlockStmt {
                stmts: vec![Stmt::Return(ReturnStmt {
                    arg: Some(Box::new(expr.clone())),
                    ..Default::default()
                })],
                ..Default::default()
            }),
            ..Default::default()
        }
        .into()
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
                        ImportMember::Named(ImportNamedMember {
                            ident,
                            alias: Some(alias_ident),
                        }) => kv_prop(&ident.sym, alias_ident.clone().into()),
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
                        ImportMember::Named(ImportNamedMember { ident, alias: None }) => {
                            PropOrSpread::Prop(Box::new(Prop::Shorthand(ident.clone())))
                        }
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
                        ImportMember::Namespace(ImportNamespaceMember { mod_ident, .. }) => {
                            spread_prop(mod_ident.clone().into())
                        }
                    })
                    .collect::<Vec<PropOrSpread>>();

                self.to_lazy(&obj_lit_expr(props))
            }
            ModuleRef::DynImport(dyn_imp) => self.to_lazy(&dyn_imp.orig_expr),
            ModuleRef::Require(require) => self.to_lazy(&require.orig_expr),
        }
    }

    /// Wraps the given expression with a __exports.ns function call expression.
    ///
    /// ```js
    /// __exports.ns(<expr>);
    /// ```
    fn to_ns_export(&self, expr: Expr) -> Expr {
        self.exports_ident
            .clone()
            .make_member(quote_ident!("ns"))
            .as_call(DUMMY_SP, vec![expr.into()])
    }

    /// ```js
    /// var { foo, bar, default: baz } = __require('./foo');
    /// ```
    fn assign_required_deps_stmt(&self, src: &Atom, exp_ident: &Ident) -> Stmt {
        self.require_call(src)
            .make_assign_to(
                AssignOp::Assign,
                AssignTarget::Simple(SimpleAssignTarget::Ident(exp_ident.clone().into())),
            )
            .into_stmt()
    }

    /// Returns a list of require call expressions that reference modules from global registry.
    ///
    /// ```js
    /// // Examples
    /// var { default: React, useState, useCallback } = __require('react');
    /// var { core } = __require('@app/core');
    /// __x = __require('./foo');
    /// __x1 = __require('./bar');
    /// __x2 = __require('./baz');
    /// ```
    fn to_require_dep_stmts(&self, src: &Atom, module_ref: &ModuleRef) -> Option<Vec<Stmt>> {
        let mut requires: Vec<Stmt> = Vec::new();
        let mut dep_props: Vec<ObjectPatProp> = Vec::new();

        match module_ref {
            ModuleRef::Import(ImportRef { members }) => {
                members
                    .iter()
                    .for_each(|module_member| match module_member {
                        ImportMember::Named(ImportNamedMember {
                            ident,
                            alias: Some(alias_ident),
                        }) => dep_props.push(ObjectPatProp::KeyValue(KeyValuePatProp {
                            key: PropName::Ident(ident.clone().into()),
                            value: Box::new(Pat::Ident(alias_ident.clone().into())),
                        })),
                        ImportMember::Named(ImportNamedMember { ident, alias: None }) => {
                            dep_props.push(ObjectPatProp::Assign(AssignPatProp {
                                key: ident.clone().into(),
                                value: None,
                                span: DUMMY_SP,
                            }));
                        }
                        ImportMember::Namespace(ImportNamespaceMember {
                            ident: Some(ident),
                            ..
                        }) => requires.push(
                            self.decl_required_deps_stmt(src, Pat::Ident(ident.clone().into())),
                        ),
                        ImportMember::Namespace(ImportNamespaceMember {
                            export_ident,
                            ident: None,
                            ..
                        }) => requires.push(self.assign_required_deps_stmt(src, &export_ident)),
                    });
            }
            // Skips AST generation because it has already been replaced during the visit phases.
            ModuleRef::DynImport(_) | ModuleRef::Require(_) => return None,
        };

        if dep_props.len() > 0 {
            requires.push(
                self.decl_required_deps_stmt(
                    src,
                    ObjectPat {
                        props: dep_props,
                        optional: false,
                        type_ann: None,
                        span: DUMMY_SP,
                    }
                    .into(),
                ),
            );
        }

        requires.into()
    }
}

impl<'a> ModuleCollector<'a> {
    pub fn new(exports_ident: &'a Ident, require_ident: &'a Ident) -> Self {
        ModuleCollector {
            exports_ident,
            require_ident,
            mods: AHashMap::default(),
            mods_idx: Vec::default(),
            exps: Vec::default(),
        }
    }
}

impl<'a> VisitMut for ModuleCollector<'a> {
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
                ModuleItem::ModuleDecl(module_decl) => {
                    module_decl.visit_mut_children_with(self);

                    match module_decl {
                        // Import statements.
                        //
                        // ```js
                        // import foo from './foo';
                        // import { foo } from './foo';
                        // import { foo as bar } from './foo';
                        // import * as foo from './foo';
                        // ```
                        ModuleDecl::Import(import) => {
                            let members = self.to_import_members(&import.specifiers);
                            let src = import.src.value.clone();

                            self.register_mod_by_members(&src, members);
                        }
                        // Named export statements with declarations.
                        //
                        // ```js
                        // export const foo = ...;
                        // export function foo() { ... }
                        // export class Foo { ... }
                        // ```
                        ModuleDecl::ExportDecl(export_decl) => {
                            let (orig_ident, decl_expr) = get_expr_from_decl(&export_decl.decl);
                            let member = ExportMember::anonymous(orig_ident.sym);

                            *item = assign_expr(&member.export_ident, decl_expr)
                                .into_stmt()
                                .into();

                            self.exps
                                .push(ExportRef::Named(NamedExportRef::new(vec![member])));
                        }
                        // Default export statements with declarations.
                        //
                        // ```js
                        // export default function foo() { ... }
                        // export default class Foo { ... }
                        // ```
                        ModuleDecl::ExportDefaultDecl(export_default_decl) => {
                            let member = ExportMember::anonymous(Atom::new("default"));

                            // Rewrite exports statements to `__x = <...>;`
                            // and register `__x` to export refs.
                            *item = assign_expr(
                                &member.export_ident,
                                get_expr_from_default_decl(&export_default_decl.decl),
                            )
                            .into_stmt()
                            .into();

                            self.exps
                                .push(ExportRef::Named(NamedExportRef::new(vec![member])));
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
                        ModuleDecl::ExportNamed(export_named) => {
                            match &export_named.src {
                                // Re-exports
                                Some(src_str) => {
                                    let src: Atom = src_str.clone().value;
                                    let import_member = ImportNamespaceMember::anonymous();
                                    let specifier = export_named.specifiers.get(0).unwrap();

                                    if let Some(ns_specifier) = specifier.as_namespace() {
                                        // Re-export all with alias.
                                        // In this case, the size of the `specifier` vector is always 1.
                                        //
                                        // ```js
                                        // export * as foo from './foo';
                                        // ```
                                        self.reg_export(ExportRef::ReExportAll(
                                            ReExportAllRef::new(
                                                &import_member.mod_ident,
                                                &import_member.export_ident,
                                                &src,
                                                Some(ns_specifier.name.atom().clone()),
                                            ),
                                        ));
                                    } else {
                                        // Re-export specified members only.
                                        //
                                        // ```js
                                        // export { foo, bar as baz } from './foo';
                                        // export { default } from './foo';
                                        // export { default as named } from './foo';
                                        // ```
                                        self.reg_export(ExportRef::NamedReExport(
                                            NamedReExportRef::new(
                                                &import_member.mod_ident,
                                                &import_member.export_ident,
                                                &src,
                                                self.to_export_members(&export_named.specifiers),
                                            ),
                                        ));
                                    }

                                    // Register new import for reference re-exported modules.
                                    //
                                    // ```js
                                    // // Before
                                    // export * as foo from './foo';
                                    //
                                    // // After
                                    // import * as __x from './foo';
                                    //
                                    // __x; // can be referenced.
                                    //
                                    // export { __x as foo };
                                    // ```
                                    self.register_mod_by_members(
                                        &src,
                                        vec![ImportMember::Namespace(import_member)],
                                    );
                                }
                                // Named export
                                None => {
                                    let members = self.to_export_members(&export_named.specifiers);
                                    self.reg_export(ExportRef::Named(NamedExportRef::new(members)));
                                }
                            }
                        }
                        // Re-exports all statements.
                        //
                        // ```js
                        // export * from './foo';
                        // ```
                        ModuleDecl::ExportAll(ExportAll {
                            src,
                            type_only: false,
                            with: None,
                            ..
                        }) => {
                            let src = src.clone().value;
                            let member = ImportNamespaceMember::anonymous();

                            self.reg_export(ExportRef::ReExportAll(ReExportAllRef::new(
                                &member.mod_ident,
                                &member.export_ident,
                                &src,
                                None,
                            )));

                            self.register_mod_by_members(
                                &src,
                                vec![ImportMember::Namespace(member)],
                            );
                        }
                        // Default export statements.
                        //
                        // ```js
                        // export default <expr>;
                        // ```
                        ModuleDecl::ExportDefaultExpr(ExportDefaultExpr { expr, .. }) => {
                            let orig_expr = *expr.clone();
                            let member = ExportMember::anonymous(Atom::new("default"));

                            *item = assign_expr(&member.export_ident, orig_expr)
                                .into_stmt()
                                .into();

                            self.reg_export(ExportRef::Named(NamedExportRef::new(vec![member])));
                        }
                        _ => {}
                    }
                }
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
                            self.register_mod(
                                &str.value,
                                ModuleRef::Require(RequireRef::new(&orig_expr)),
                            );
                            *expr = self.require_call(&str.value);
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
                            self.register_mod(
                                &str.value,
                                ModuleRef::DynImport(DynImportRef::new(&orig_expr)),
                            );
                            *expr = self.require_call(&str.value);
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
