use std::mem;

use crate::{
    models::*,
    phase::ModulePhase,
    utils::{
        ast::{
            assign_expr, import_star, kv_prop, num_lit_expr, obj_lit_expr, spread_prop,
            var_declarator,
        },
        parse::{get_expr_from_decl, get_expr_from_default_decl},
    },
};
use swc_core::{
    atoms::Atom,
    common::{collections::AHashMap, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{private_ident, quote_ident, ExprFactory},
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
};

pub struct GlobalModuleTransformer {
    pub id: u32,
    pub deps_id: Option<AHashMap<String, u32>>,
    pub phase: ModulePhase,
    ctx_ident: Ident,
    // `import ... from './foo'`;
    // `require('./foo')`;
    //
    // key: './foo'
    // value: Dep
    mods: AHashMap<Atom, ModuleRef>,
    mods_idx: Vec<Atom>,
    exports: Vec<ExportRef>,
}

impl GlobalModuleTransformer {
    /// Returns the AST structure based on the collected module and export data.
    pub fn get_module_body(&self, orig_body: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let (imports, body, exports) = self.partition_by_module_item(orig_body);
        let (ctx_import, ctx_decl) = self.get_ctx_ast();

        // Object properties to be passed to the Global module's export API."
        let mut export_props: Vec<PropOrSpread> = Vec::new();

        // An import statement newly added by the re-exports.
        let mut additional_imports: Vec<ModuleItem> = Vec::new();

        // A statements that retrieves injected dependencies
        // through the Global Module's require API.
        //
        // ```js
        // var { ... } = __require('./foo');
        // var { ... } = __require('./bar');
        // ```
        let mut deps_requires: Vec<ModuleItem> = Vec::new();

        // An expressions that assigns the module reference value to the export variable.
        //
        // ```js
        // __x = foo;
        // __x1 = bar;
        // __x2 = bar;
        // ```
        let mut export_assigns: Vec<ModuleItem> = Vec::new();

        // A list of export variable declarators,
        //
        // ```js
        // var __x, __x1, __x2;
        // // => __x, __x1, __x2
        // ```
        let mut export_decls: Vec<VarDeclarator> = Vec::new();

        self.mods_idx.iter().for_each(|src| {
            let value = self.mods.get(src).unwrap();
            if let Some(stmts) = self.to_require_dep_stmts(src, value) {
                deps_requires.extend(stmts);
            }
        });

        self.exports.iter().for_each(|export_ref| match export_ref {
            ExportRef::Named(NamedExportRef { members }) => members.iter().for_each(|member| {
                if let Some(orig_ident) = &member.orig_ident {
                    export_assigns.push(
                        orig_ident
                            .clone()
                            .make_assign_to(AssignOp::Assign, member.export_ident.clone().into())
                            .into_stmt()
                            .into(),
                    );
                }

                export_props.push(kv_prop(&member.name, member.export_ident.clone().into()));
                export_decls.push(var_declarator(&member.export_ident));
            }),
            ExportRef::NamedReExport(NamedReExportRef {
                mod_ident,
                exp_ident,
                src,
                members,
            }) => {
                additional_imports.push(import_star(mod_ident, src));
                export_props.extend(members.iter().map(|member| {
                    kv_prop(
                        &member.name,
                        exp_ident
                            .clone()
                            .make_member(member.name.clone().into())
                            .into(),
                    )
                }));
                export_decls.push(var_declarator(&exp_ident));
            }
            ExportRef::ReExportAll(ReExportAllRef {
                mod_ident,
                exp_ident,
                src,
                name,
            }) => {
                additional_imports.push(import_star(mod_ident, src));
                export_props.push(match name {
                    Some(exp_name) => kv_prop(exp_name, exp_ident.clone().into()),
                    None => spread_prop(self.to_ns_export(exp_ident.clone().into())),
                });
                export_decls.push(var_declarator(&exp_ident));
            }
        });

        let exports_call = self.exports_call(obj_lit_expr(export_props));
        let export_vars = VarDecl {
            kind: VarDeclKind::Var,
            decls: export_decls,
            ..Default::default()
        };

        let mut new_body = vec![];

        // Imports
        new_body.push(ctx_import);
        new_body.extend(imports);
        new_body.extend(additional_imports);

        // Body
        new_body.push(ctx_decl);
        new_body.extend(deps_requires);
        new_body.extend(body);
        new_body.push(exports_call.into_stmt().into());
        new_body.push(export_vars.into());

        // Exports
        new_body.extend(exports);

        new_body
    }

    fn partition_by_module_item(
        &self,
        orig_body: Vec<ModuleItem>,
    ) -> (Vec<ModuleItem>, Vec<ModuleItem>, Vec<ModuleItem>) {
        let size = orig_body.len();
        let mut imports = Vec::with_capacity(size);
        let mut exports = Vec::with_capacity(size);
        let mut body = Vec::with_capacity(size);

        orig_body.into_iter().for_each(|item| match item {
            ModuleItem::Stmt(_) => body.push(item),
            ModuleItem::ModuleDecl(ref module_decl) => match module_decl {
                ModuleDecl::Import(_) => imports.push(item),
                _ => exports.push(item),
            },
        });

        (imports, body, exports)
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
        if self.mods.get(src).is_none() {
            self.insert_mod(src, module_ref);
        }
    }

    /// Register export reference.
    fn register_export(&mut self, export: ExportRef) {
        self.exports.push(export);
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
            ImportSpecifier::Named(
                specifier @ ImportNamedSpecifier {
                    is_type_only: false,
                    ..
                },
            ) => members.push(specifier.into()),
            ImportSpecifier::Namespace(specifier) => members.push(specifier.into()),
            ImportSpecifier::Default(specifier) => members.push(specifier.into()),
            _ => {}
        });

        members
    }

    /// Convert `ExportSpecifier` into `ExportMember`.
    fn to_export_members(&self, specifiers: &Vec<ExportSpecifier>) -> Vec<ExportMember> {
        let mut members = Vec::with_capacity(specifiers.len());

        specifiers.iter().for_each(|spec| match spec {
            ExportSpecifier::Named(
                specifier @ ExportNamedSpecifier {
                    is_type_only: false,
                    ..
                },
            ) => members.push(specifier.into()),
            _ => {}
        });

        members
    }

    /* ----- AST Helpers ----- */

    fn get_ctx_ast(&self) -> (ModuleItem, ModuleItem) {
        let import_member = private_ident!("__c");
        let import_stmt = ModuleDecl::Import(ImportDecl {
            specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                imported: Some(ModuleExportName::Ident(
                    quote_ident!(match self.phase {
                        ModulePhase::Register => "register",
                        ModulePhase::Runtime => "getContext",
                    })
                    .into(),
                )),
                is_type_only: false,
                local: import_member.clone(),
                span: DUMMY_SP,
            })],
            src: Box::new("@global-modules/runtime".into()),
            phase: ImportPhase::Evaluation,
            type_only: false,
            with: None,
            span: DUMMY_SP,
        });

        let ctx_decl = import_member
            .as_call(DUMMY_SP, vec![num_lit_expr(self.id).as_arg()])
            .into_var_decl(VarDeclKind::Var, self.ctx_ident.clone().into());

        (import_stmt.into(), ctx_decl.into())
    }

    /// Returns global module's require call expression.
    ///
    /// ```js
    /// ctx.require(src);
    /// ```
    fn require_call(&self, src: &Atom, initial_value: Option<Expr>) -> Expr {
        let mut args = vec![src.clone().as_arg()];

        if let Some(expr) = initial_value {
            args.push(expr.as_arg());
        }

        self.ctx_ident
            .clone()
            .make_member(quote_ident!("require"))
            .as_call(DUMMY_SP, vec![src.clone().as_arg()])
    }

    /// Returns global module's exports call expression.
    ///
    /// ```js
    /// ctx.exports({ ... });
    /// ```
    fn exports_call(&self, obj: Expr) -> Expr {
        self.ctx_ident
            .clone()
            .make_member(quote_ident!("exports"))
            .as_call(DUMMY_SP, vec![obj.into()])
    }

    /// ```js
    /// var { foo, bar, default: baz } = __require('./foo');
    /// ```
    fn decl_required_deps_stmt(&self, src: &Atom, pat: Pat) -> Stmt {
        self.require_call(src, None)
            .into_var_decl(VarDeclKind::Var, pat)
            .into()
    }

    /// Wraps the given expression with a __exports.ns function call expression.
    ///
    /// ```js
    /// __exports.ns(<expr>);
    /// ```
    fn to_ns_export(&self, expr: Expr) -> Expr {
        Ident::from(quote_ident!("ns")) // TODO
            .make_member(quote_ident!("ns"))
            .as_call(DUMMY_SP, vec![expr.into()])
    }

    /// ```js
    /// var { foo, bar, default: baz } = __require('./foo');
    /// ```
    fn assign_required_deps_stmt(&self, src: &Atom, exp_ident: &Ident) -> Stmt {
        self.require_call(src, None)
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
    fn to_require_dep_stmts(&self, src: &Atom, module_ref: &ModuleRef) -> Option<Vec<ModuleItem>> {
        let mut requires: Vec<ModuleItem> = Vec::new();
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
                            self.decl_required_deps_stmt(src, Pat::Ident(ident.clone().into()))
                                .into(),
                        ),
                        ImportMember::Namespace(ImportNamespaceMember {
                            export_ident,
                            ident: None,
                            ..
                        }) => {
                            requires.push(self.assign_required_deps_stmt(src, &export_ident).into())
                        }
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
                )
                .into(),
            );
        }

        requires.into()
    }
}

impl GlobalModuleTransformer {
    pub fn new(id: u32, phase: ModulePhase, deps_id: Option<AHashMap<String, u32>>) -> Self {
        GlobalModuleTransformer {
            id,
            deps_id,
            phase,
            ctx_ident: private_ident!("__ctx"),
            mods: AHashMap::default(),
            mods_idx: Vec::default(),
            exports: Vec::default(),
        }
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
                        ModuleDecl::Import(import) => import.visit_mut_children_with(self),
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

                            self.register_export(ExportRef::Named(NamedExportRef::new(vec![
                                member,
                            ])));
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

                            self.register_export(ExportRef::Named(NamedExportRef::new(vec![
                                member,
                            ])));
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
                                        self.register_export(ExportRef::ReExportAll(
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
                                        self.register_export(ExportRef::NamedReExport(
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
                                    self.register_export(ExportRef::Named(NamedExportRef::new(
                                        members,
                                    )));
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

                            self.register_export(ExportRef::ReExportAll(ReExportAllRef::new(
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

                            self.register_export(ExportRef::Named(NamedExportRef::new(vec![
                                member,
                            ])));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn visit_mut_import_decl(&mut self, import_decl: &mut ImportDecl) {
        if false {
            return;
        }

        let members = self.to_import_members(&import_decl.specifiers);
        let src = import_decl.src.value.clone();

        self.register_mod_by_members(&src, members);
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
                            *expr = self.require_call(&str.value, None);
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
                            *expr = self.require_call(&str.value, None);
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

// export { a as b };
// "type": "ExportNamedDeclaration"
// ExportSpecifier
// orig: a
// exported b

// "type": "ExportNamedDeclaration",
// export { foo as bar } from './baz';
// ExportSpecifier
// orig: foo
// exported bar
// source: "./baz"
