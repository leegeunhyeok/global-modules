use std::mem;

use crate::{
    models::*,
    phase::ModulePhase,
    utils::{
        ast::{
            assign_expr, import_star, kv_prop, num_lit_expr, obj_lit_expr, spread_prop,
            var_declarator,
        },
        collections::OHashMap,
        parse::{get_expr_from_decl, get_expr_from_default_decl},
    },
};
use swc_core::{
    atoms::Atom,
    common::{collections::AHashMap, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{member_expr, private_ident, quote_ident, ExprFactory},
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
    deps: OHashMap<Atom, ModuleRef>,
    exports: Vec<ExportRef>,
    stmts: Vec<Stmt>,
}

impl GlobalModuleTransformer {
    /// Returns the AST structure based on the collected module and export data.
    pub fn get_module_body(&mut self, orig_body: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let (imports, body, exports) = self.partition_by_module_item(orig_body);

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

        // A list of binding variable declarators,
        //
        // ```js
        // var __x, __x1, __x2;
        // // => __x, __x1, __x2
        // ```
        let mut export_decls: Vec<VarDeclarator> = Vec::new();

        self.deps.iter().for_each(|(src, value)| {
            deps_requires.extend(self.to_require_dep_stmts(src, value));
        });

        self.exports.iter().for_each(|export_ref| match export_ref {
            ExportRef::Named(NamedExportRef { members }) => {
                members.iter().for_each(|member| match member {
                    ExportMember::Actual(actual_export) => {
                        let name = actual_export
                            .name
                            .as_ref()
                            .map_or_else(|| actual_export.ident.sym.clone(), |name| name.clone());

                        export_props.push(kv_prop(&name, actual_export.ident.clone().into()));
                    }
                    ExportMember::Binding(binding_export) => {
                        export_props.push(kv_prop(
                            &binding_export.name,
                            binding_export.bind_ident.clone().into(),
                        ));
                        export_decls.push(var_declarator(&binding_export.bind_ident));
                    }
                })
            }
            ExportRef::NamedReExport(NamedReExportRef {
                mod_ident,
                src,
                members,
            }) => {
                if self.phase == ModulePhase::Register {
                    additional_imports.push(import_star(mod_ident, src));
                }

                export_props.extend(members.iter().map(|member| {
                    match member {
                        ExportMember::Actual(actual_export) => {
                            let name = actual_export.name.as_ref().map_or_else(
                                || actual_export.ident.sym.clone(),
                                |name| name.clone(),
                            );

                            kv_prop(
                                &name,
                                mod_ident
                                    .clone()
                                    .make_member(actual_export.ident.sym.clone().into())
                                    .into(),
                            )
                        }
                        ExportMember::Binding(binding_export) => kv_prop(
                            &binding_export.name,
                            mod_ident
                                .clone()
                                .make_member(binding_export.name.clone().into())
                                .into(),
                        ),
                    }
                }));
            }
            ExportRef::ReExportAll(ReExportAllRef {
                src,
                mod_ident,
                name,
            }) => {
                if self.phase == ModulePhase::Register {
                    additional_imports.push(import_star(mod_ident, src));
                }

                let ns_call = self.to_ns_export(mod_ident.clone().into());

                match name {
                    Some(exp_name) => export_props.push(kv_prop(exp_name, ns_call)),
                    None => export_props.insert(0, spread_prop(ns_call)),
                };
            }
        });

        let exports_call = self.exports_call(obj_lit_expr(export_props));
        let exports_decl = VarDecl {
            kind: VarDeclKind::Var,
            decls: export_decls,
            ..Default::default()
        };

        let mut new_body = vec![];

        // Imports
        if self.phase == ModulePhase::Register {
            let (import_stmt, ident) = self.global_module_import_stmt();
            new_body.push(import_stmt);
            new_body.extend(imports);
            new_body.extend(additional_imports);
            new_body.push(
                ident
                    .as_call(DUMMY_SP, vec![num_lit_expr(self.id).as_arg()])
                    .into_var_decl(VarDeclKind::Var, self.ctx_ident.clone().into())
                    .into(),
            );
        }

        // Body
        if self.phase == ModulePhase::Runtime {
            new_body.push(
                // `global.__modules.getContext(id);`
                member_expr!(Default::default(), DUMMY_SP, global.__modules.getContext)
                    .as_call(DUMMY_SP, vec![num_lit_expr(self.id).as_arg()])
                    .into_var_decl(VarDeclKind::Var, self.ctx_ident.clone().into())
                    .into(),
            );
            new_body.extend(deps_requires);
        }
        let stmts = mem::take(&mut self.stmts);
        new_body.extend(body);
        new_body.extend(stmts.into_iter().map(|stmt| stmt.into()));
        new_body.push(exports_call.into_stmt().into());
        new_body.push(exports_decl.into());

        // Exports
        if self.phase == ModulePhase::Register {
            new_body.extend(exports);
        }

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
            // Always collect statements.
            ModuleItem::Stmt(_) => body.push(item),
            // Collects import and export statements during the ModulePhase::Register phase.
            ModuleItem::ModuleDecl(ref module_decl) if self.phase == ModulePhase::Register => {
                match module_decl {
                    ModuleDecl::Import(_) => imports.push(item),
                    _ => exports.push(item),
                }
            }
            _ => {}
        });

        (imports, body, exports)
    }

    /// Register additional statements.
    fn register_stmt(&mut self, stmt: Stmt) {
        self.stmts.push(stmt);
    }

    /// Register module reference by members.
    fn register_mod(&mut self, src: &Atom, members: Vec<ImportMember>) {
        if let Some(mod_ref) = self.deps.get_mut(&src) {
            mod_ref.members.extend(members.into_iter());
        } else {
            self.deps.insert(src, ModuleRef::new(members));
        }
    }

    /// Register export reference.
    fn register_export(&mut self, export: ExportRef) {
        self.exports.push(export);
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

    /// ```js
    /// // Code
    /// // value.0
    /// import { register as __r } from '@global-modules/runtime';
    ///
    /// // value.1
    /// __r
    /// ```
    fn global_module_import_stmt(&self) -> (ModuleItem, Ident) {
        let import_ident = private_ident!("__r");
        let import_stmt = ModuleDecl::Import(ImportDecl {
            specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                imported: Some(ModuleExportName::Ident(quote_ident!("register").into())),
                is_type_only: false,
                local: import_ident.clone(),
                span: DUMMY_SP,
            })],
            src: Box::new("@global-modules/runtime".into()),
            phase: ImportPhase::Evaluation,
            type_only: false,
            with: None,
            span: DUMMY_SP,
        });

        (import_stmt.into(), import_ident)
    }

    /// Returns global module's require call expression.
    ///
    /// ```js
    /// // Code
    /// ctx.require(src);
    /// ```
    fn require_call(&self, src: &Atom) -> Expr {
        self.ctx_ident
            .clone()
            .make_member(quote_ident!("require"))
            .as_call(DUMMY_SP, vec![src.clone().as_arg()])
    }

    /// Returns global module's exports call expression.
    ///
    /// ```js
    /// // Code
    /// ctx.exports(function () {
    ///   return <obj>;
    /// });
    /// ```
    fn exports_call(&self, obj: Expr) -> Expr {
        self.ctx_ident
            .clone()
            .make_member(quote_ident!("exports"))
            .as_call(DUMMY_SP, vec![obj.into_lazy_fn(vec![]).as_arg()])
    }

    /// ```js
    /// // Code
    /// var { foo, bar, default: baz } = __require('./foo');
    /// ```
    fn decl_required_deps_stmt(&self, src: &Atom, pat: Pat) -> Stmt {
        self.require_call(src)
            .into_var_decl(VarDeclKind::Var, pat)
            .into()
    }

    /// Wraps the given expression with a `__ctx.ns` function call expression.
    ///
    /// ```js
    /// // Code
    /// __ctx.ns(<expr>);
    /// ```
    fn to_ns_export(&self, expr: Expr) -> Expr {
        self.ctx_ident
            .clone()
            .make_member(quote_ident!("ns"))
            .as_call(DUMMY_SP, vec![expr.into()])
    }

    /// Returns a list of require call expressions that reference modules from global registry.
    ///
    /// ```js
    /// // Code
    /// var { default: React, useState, useCallback } = __require('react');
    /// var { core } = __require('@app/core');
    /// var ns = __require('@app/internal');
    /// ```
    fn to_require_dep_stmts(&self, src: &Atom, module_ref: &ModuleRef) -> Vec<ModuleItem> {
        let mut requires = Vec::new();
        let mut dep_props = Vec::new();

        module_ref
            .members
            .iter()
            .for_each(|module_member| match module_member {
                ImportMember::Default(ImportDefaultMember { ident, .. }) => {
                    dep_props.push(ObjectPatProp::KeyValue(KeyValuePatProp {
                        key: PropName::Ident(quote_ident!("default").into()),
                        value: Box::new(Pat::Ident(ident.clone().into())),
                    }))
                }
                ImportMember::Named(ImportNamedMember {
                    ident,
                    alias: Some(alias_ident),
                    ..
                }) => dep_props.push(ObjectPatProp::KeyValue(KeyValuePatProp {
                    key: PropName::Ident(ident.clone().into()),
                    value: Box::new(Pat::Ident(alias_ident.clone().into())),
                })),
                ImportMember::Named(ImportNamedMember {
                    ident, alias: None, ..
                }) => {
                    dep_props.push(ObjectPatProp::Assign(AssignPatProp {
                        key: ident.clone().into(),
                        value: None,
                        span: DUMMY_SP,
                    }));
                }
                ImportMember::Namespace(ImportNamespaceMember { ident, .. }) => requires.push(
                    self.decl_required_deps_stmt(src, ident.clone().into())
                        .into(),
                ),
            });

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

        requires
    }
}

impl GlobalModuleTransformer {
    pub fn new(id: u32, phase: ModulePhase, deps_id: Option<AHashMap<String, u32>>) -> Self {
        GlobalModuleTransformer {
            id,
            deps_id,
            phase,
            ctx_ident: private_ident!("__ctx"),
            deps: OHashMap::default(),
            exports: Vec::default(),
            stmts: Vec::default(),
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
                            let binding_export = BindingExportMember::new(orig_ident.sym.clone());

                            *item = ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
                                specifiers: vec![ExportSpecifier::Named(ExportNamedSpecifier {
                                    orig: ModuleExportName::Ident(
                                        binding_export.bind_ident.clone(),
                                    ),
                                    exported: Some(ModuleExportName::Ident(orig_ident.clone())),
                                    is_type_only: false,
                                    span: DUMMY_SP,
                                })],
                                src: None,
                                with: None,
                                type_only: false,
                                span: DUMMY_SP,
                            }));

                            self.register_stmt(
                                assign_expr(&binding_export.bind_ident, decl_expr)
                                    .into_stmt()
                                    .into(),
                            );
                            self.register_export(ExportRef::Named(NamedExportRef::new(vec![
                                ExportMember::Binding(binding_export),
                            ])));
                        }
                        // Default export statements with declarations.
                        //
                        // ```js
                        // export default function foo() { ... }
                        // export default class Foo { ... }
                        // ```
                        ModuleDecl::ExportDefaultDecl(export_default_decl) => {
                            let binding_export = BindingExportMember::new("default".into());

                            // Rewrite exports statements to `__x = <decl>;` and register `__x`.
                            *item = ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
                                ExportDefaultExpr {
                                    expr: assign_expr(
                                        &binding_export.bind_ident,
                                        get_expr_from_default_decl(&export_default_decl.decl),
                                    )
                                    .into(),
                                    span: DUMMY_SP,
                                },
                            ));

                            self.register_export(ExportRef::Named(NamedExportRef::new(vec![
                                ExportMember::Binding(binding_export),
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
                                    let src = src_str.clone().value;
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
                                                &import_member.ident,
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
                                                &import_member.ident,
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
                                    self.register_mod(
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
                            let import_member = ImportNamespaceMember::anonymous();

                            self.register_export(ExportRef::ReExportAll(ReExportAllRef::new(
                                &import_member.ident,
                                &src,
                                None,
                            )));

                            self.register_mod(&src, vec![ImportMember::Namespace(import_member)]);
                        }
                        // Default export statements.
                        //
                        // ```js
                        // export default <expr>;
                        // ```
                        ModuleDecl::ExportDefaultExpr(export_default_decl) => {
                            let orig_expr = (*export_default_decl.expr).clone();
                            let binding_export = BindingExportMember::new("default".into());

                            export_default_decl.expr =
                                assign_expr(&binding_export.bind_ident, orig_expr).into();

                            self.register_export(ExportRef::Named(NamedExportRef::new(vec![
                                ExportMember::Binding(binding_export),
                            ])));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn visit_mut_import_decl(&mut self, import_decl: &mut ImportDecl) {
        if self.phase == ModulePhase::Register {
            return;
        }

        let members = self.to_import_members(&import_decl.specifiers);
        let src = import_decl.src.value.clone();

        self.register_mod(&src, members);
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Call(call_expr) => match call_expr {
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
                } if self.phase == ModulePhase::Runtime
                    && args.len() == 1
                    && callee_expr.is_ident_ref_to("require") =>
                {
                    let src = args.get(0).unwrap();

                    match &*src.expr {
                        // The first argument of the `require` function must be a string type only.
                        Expr::Lit(Lit::Str(str)) => {
                            *expr = self.require_call(&str.value);
                        }
                        _ => panic!("invalid `require` call expression"),
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
                } if self.phase == ModulePhase::Runtime => {
                    let src = args.get(0).expect("invalid dynamic import call");

                    match &*src.expr {
                        // The first argument of the `import` function must be a string type only.
                        Expr::Lit(Lit::Str(str)) => {
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
