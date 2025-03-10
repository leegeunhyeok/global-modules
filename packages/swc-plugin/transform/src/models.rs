use std::default;

use presets::{decl_require_deps_stmt, require_call};
use swc_core::{
    atoms::Atom,
    common::{Spanned, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{private_ident, ExprFactory},
    },
    plugin::errors::HANDLER,
};
use tracing::debug;

use crate::{phase::ModulePhase, utils::ast::*};

#[derive(Debug)]
pub struct ModuleRef {
    // `import def, { foo, bar as baz } from '...'`;
    // => def, foo, bar (alias: baz)
    pub members: Vec<ImportMember>,
}

impl ModuleRef {
    pub fn new(members: Vec<ImportMember>) -> Self {
        ModuleRef { members }
    }
}

#[derive(Debug)]
pub enum ImportMember {
    Default(ImportDefaultMember),
    Named(ImportNamedMember),
    Namespace(ImportNamespaceMember),
}

impl From<&ImportDefaultSpecifier> for ImportMember {
    fn from(value: &ImportDefaultSpecifier) -> Self {
        ImportMember::Default(ImportDefaultMember::new(&value.local))
    }
}

impl From<&ImportNamedSpecifier> for ImportMember {
    fn from(value: &ImportNamedSpecifier) -> Self {
        if let Some(ModuleExportName::Ident(ident)) = &value.imported {
            ImportMember::Named(ImportNamedMember::new(&ident, Some(value.local.clone())))
        } else {
            ImportMember::Named(ImportNamedMember::new(&value.local, None))
        }
    }
}

impl From<&ImportStarAsSpecifier> for ImportMember {
    fn from(value: &ImportStarAsSpecifier) -> Self {
        ImportMember::Namespace(ImportNamespaceMember::new(&value.local))
    }
}

#[derive(Debug)]
pub struct ImportDefaultMember {
    // `import foo from 'foo';`
    // => foo
    pub ident: Ident,
}

impl ImportDefaultMember {
    pub fn new(ident: &Ident) -> Self {
        ImportDefaultMember {
            ident: ident.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ImportNamedMember {
    // `import { foo } from 'foo';`
    // `import * as foo from 'foo';`
    // => foo
    pub ident: Ident,
    // `import { foo as bar } from 'foo'`;
    // => bar
    pub alias: Option<Ident>,
}

impl ImportNamedMember {
    pub fn new(ident: &Ident, alias: Option<Ident>) -> Self {
        ImportNamedMember {
            ident: ident.clone(),
            alias,
        }
    }
}

#[derive(Debug)]
pub struct ImportNamespaceMember {
    /// A reference identifier for the import statement used in re-export handling.
    ///
    /// ```js
    /// import * as foo from './foo';
    /// // => foo
    /// ```
    pub ident: Ident,
}

impl ImportNamespaceMember {
    pub fn new(ident: &Ident) -> Self {
        Self {
            ident: ident.clone(),
        }
    }
}

#[derive(Debug)]
pub enum ExportRef {
    /// ```js
    /// export { foo, bar };
    /// export const ...;
    /// export function ...;
    /// export class ...;
    /// export default ...; // named as 'default'.
    /// ```
    Named(NamedExportRef),
    /// ```js
    /// export { foo, bar as baz } from './foo';
    /// export { default } from './foo';
    /// export { default as foo } from './foo';
    /// ```
    NamedReExport(NamedReExportRef),
    /// ```js
    /// export * from './foo';
    /// export * as foo from './foo';
    /// ```
    ReExportAll(ReExportAllRef),
}

#[derive(Debug)]
pub struct NamedExportRef {
    pub members: Vec<ExportMember>,
}

impl NamedExportRef {
    pub fn new(members: Vec<ExportMember>) -> Self {
        NamedExportRef { members }
    }
}

#[derive(Debug)]
pub enum ExportMember {
    Actual(ActualExportMember),
    Binding(BindingExportMember),
}

impl From<&ExportNamedSpecifier> for ExportMember {
    fn from(value: &ExportNamedSpecifier) -> Self {
        match &value.orig {
            ModuleExportName::Ident(orig_ident) => ExportMember::Actual(ActualExportMember::new(
                &orig_ident,
                if let Some(ModuleExportName::Ident(exported_ident)) = &value.exported {
                    Some(exported_ident.sym.clone())
                } else {
                    None
                },
            )),
            ModuleExportName::Str(_) => HANDLER.with(|handler| {
                handler
                    .struct_span_err(value.orig.span(), "unsupported named export specifier")
                    .emit();

                ExportMember::Actual(ActualExportMember::new(&Ident::default(), None))
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActualExportMember {
    /// The identifier of the actual export target.
    ///
    /// ```js
    /// __x = actualExportMember;
    /// // => actualExportMember
    /// ```
    pub ident: Ident,
    /// Export name.
    pub name: Option<Atom>,
}

impl ActualExportMember {
    pub fn new(orig_ident: &Ident, name: Option<Atom>) -> Self {
        Self {
            ident: orig_ident.clone(),
            name,
        }
    }

    pub fn into_bind_prop(&self, bind_ident: &Ident) -> PropOrSpread {
        let ident_sym = self.ident.sym.clone();
        let name = self.name.clone();
        let name = name.unwrap_or_else(|| ident_sym.clone());

        kv_prop(
            name,
            bind_ident.clone().make_member(ident_sym.into()).into(),
        )
    }
}

impl From<ActualExportMember> for PropOrSpread {
    fn from(value: ActualExportMember) -> PropOrSpread {
        let name = value.name.unwrap_or_else(|| value.ident.sym.clone());
        kv_prop(name, value.ident.into())
    }
}

#[derive(Clone, Debug)]
pub struct BindingExportMember {
    /// A reference variable that points to the export target.
    ///
    /// ```js
    /// __x = actualExportMember;
    /// // => __x
    /// ```
    pub bind_ident: Ident,
    /// Export name.
    pub name: Atom,
}

impl BindingExportMember {
    pub fn new(name: Atom) -> Self {
        Self {
            bind_ident: private_ident!("__x"),
            name,
        }
    }

    pub fn into_bind_prop(&self, bind_ident: &Ident) -> PropOrSpread {
        kv_prop(
            self.name.clone(),
            bind_ident
                .clone()
                .make_member(self.name.clone().into())
                .into(),
        )
    }
}

impl From<BindingExportMember> for PropOrSpread {
    fn from(value: BindingExportMember) -> PropOrSpread {
        kv_prop(value.name, value.bind_ident.into())
    }
}

impl From<BindingExportMember> for VarDeclarator {
    fn from(value: BindingExportMember) -> VarDeclarator {
        var_declarator(value.bind_ident)
    }
}

#[derive(Debug)]
pub struct NamedReExportRef {
    /// A reference identifier for the import statement used in re-export handling.
    ///
    /// ```js
    /// import * as __rx from './foo';
    /// // => __rx
    /// ```
    pub mod_ident: Ident,
    /// Source of the referenced module.
    pub src: Atom,
    pub id: Option<String>,
    /// Exported members.
    pub members: Vec<ExportMember>,
}

impl NamedReExportRef {
    pub fn new(src: Atom, id: Option<String>, members: Vec<ExportMember>) -> Self {
        NamedReExportRef {
            mod_ident: private_ident!("__mod"),
            src,
            id,
            members,
        }
    }

    pub fn get_binding_ast(&self, ctx_ident: &Ident, phase: ModulePhase) -> ModuleItem {
        match phase {
            ModulePhase::Bundle => import_star(self.mod_ident.clone(), self.src.clone()),
            ModulePhase::Runtime => decl_require_deps_stmt(
                ctx_ident,
                self.id
                    .as_ref()
                    .map(|id| Lit::from(id.as_str()))
                    .unwrap_or_else(|| Lit::from(self.src.as_str())),
                self.mod_ident.clone().into(),
            )
            .into(),
        }
    }
}

#[derive(Debug)]
pub struct ReExportAllRef {
    /// A reference identifier for the import statement used in re-export handling.
    ///
    /// ```js
    /// import * as __rx from './foo';
    /// // => __rx
    /// ```
    pub mod_ident: Ident,
    /// Source of the referenced module.
    pub src: Atom,
    pub id: Option<String>,
    /// Alias name.
    ///
    /// ```js
    /// export * as foo from './foo';
    /// // => Some(foo)
    ///
    /// export * from './foo';
    /// // => None
    /// ```
    pub name: Option<Atom>,
}

impl ReExportAllRef {
    pub fn new(src: Atom, id: Option<String>, name: Option<Atom>) -> Self {
        ReExportAllRef {
            mod_ident: private_ident!("__mod"),
            src,
            id,
            name,
        }
    }

    pub fn get_binding_ast(&self, ctx_ident: &Ident, phase: ModulePhase) -> ModuleItem {
        match phase {
            ModulePhase::Bundle => import_star(self.mod_ident.clone(), self.src.clone()),
            ModulePhase::Runtime => decl_require_deps_stmt(
                ctx_ident,
                self.id
                    .as_ref()
                    .map(|id| Lit::from(id.as_str()))
                    .unwrap_or_else(|| Lit::from(self.src.as_str())),
                self.mod_ident.clone().into(),
            )
            .into(),
        }
    }
}

pub mod helpers {
    use presets::{decl_require_deps_stmt, exports_call};
    use swc_core::{
        common::{collections::AHashMap, DUMMY_SP},
        ecma::{
            ast::*,
            utils::{quote_ident, ExprFactory},
        },
    };

    use super::*;
    use crate::{phase::ModulePhase, utils::collections::OHashMap};

    pub fn export_ref_from_named_export(
        export_named: &NamedExport,
        deps_id: &Option<AHashMap<String, String>>,
    ) -> ExportRef {
        match &export_named.src {
            // Re-exports
            Some(src_str) => {
                let src = src_str.clone().value;
                let id = deps_id
                    .as_ref()
                    .and_then(|deps_id| deps_id.get(src.as_str()));
                let specifier = export_named.specifiers.get(0).unwrap();

                if let Some(ns_specifier) = specifier.as_namespace() {
                    // Re-export all with alias.
                    // In this case, the size of the `specifier` vector is always 1.
                    //
                    // ```js
                    // export * as foo from './foo';
                    // ```
                    ExportRef::ReExportAll(ReExportAllRef::new(
                        src,
                        id.as_ref().map(|id| id.as_str().into()),
                        Some(ns_specifier.name.atom().clone()),
                    ))
                } else {
                    // Re-export specified members only.
                    //
                    // ```js
                    // export { foo, bar as baz } from './foo';
                    // export { default } from './foo';
                    // export { default as named } from './foo';
                    // ```
                    ExportRef::NamedReExport(NamedReExportRef::new(
                        src,
                        id.as_ref().map(|id| id.as_str().into()),
                        to_export_members(&export_named.specifiers),
                    ))
                }
            }
            // Named export
            None => {
                let members = to_export_members(&export_named.specifiers);
                ExportRef::Named(NamedExportRef::new(members))
            }
        }
    }

    /// Converts `ImportSpecifier` into `ImportMember`.
    pub fn to_import_members(specifiers: &Vec<ImportSpecifier>) -> Vec<ImportMember> {
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

    /// Converts the export specifiers into `ExportMember`.
    pub fn to_export_members(specifiers: &Vec<ExportSpecifier>) -> Vec<ExportMember> {
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

    /// Returns a list of require call expressions that reference modules from global registry.
    ///
    /// ```js
    /// // Code
    /// var { default: React, useState, useCallback } = global.__modules.require('react');
    /// var { core } = global.__modules.require('@app/core');
    /// var ns = global.__modules.require('@app/internal');
    /// ```
    pub fn to_require_dep_stmts(
        ctx_ident: &Ident,
        src: Lit,
        module_ref: &ModuleRef,
    ) -> Vec<ModuleItem> {
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
                    decl_require_deps_stmt(ctx_ident, src.clone(), ident.clone().into()).into(),
                ),
            });

        if dep_props.len() > 0 {
            requires.push(
                decl_require_deps_stmt(
                    ctx_ident,
                    src.clone(),
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

    /// Converts dependencies into `Vec<ModuleItem>`.
    pub fn deps_to_ast(
        ctx_ident: &Ident,
        deps: &OHashMap<Atom, ModuleRef>,
        deps_id: &Option<AHashMap<String, String>>,
    ) -> Vec<ModuleItem> {
        let mut items = vec![];

        deps.iter().for_each(|(src, value)| {
            let src_lit = if let Some(deps_id) = deps_id {
                if let Some(id) = deps_id.get(src.as_str()) {
                    Lit::Str(Str {
                        value: id.as_str().into(),
                        raw: None,
                        span: DUMMY_SP,
                    })
                    .into()
                } else {
                    None
                }
            } else {
                None
            };

            items.extend(to_require_dep_stmts(
                ctx_ident,
                src_lit.unwrap_or(Lit::from(src.as_str())),
                value,
            ));
        });

        items
    }

    /// Converts exports into `(export_bindings, export_call, export_value_decl)`.
    pub fn exports_to_ast(
        ctx_ident: &Ident,
        exports: Vec<ExportRef>,
        phase: ModulePhase,
    ) -> (Vec<ModuleItem>, Option<Stmt>, Option<Stmt>) {
        let mut export_bindings: Vec<ModuleItem> = vec![];
        let mut export_props: Vec<PropOrSpread> = vec![];
        let mut export_value_bindings: Vec<VarDeclarator> = vec![];

        exports.into_iter().for_each(|export_ref| match export_ref {
            ExportRef::Named(NamedExportRef { members }) => {
                members.into_iter().for_each(|member| match member {
                    ExportMember::Actual(actual_export) => {
                        export_props.push(actual_export.into());
                    }
                    ExportMember::Binding(binding_export) => {
                        export_props.push(binding_export.clone().into());
                        export_value_bindings.push(binding_export.into());
                    }
                })
            }
            ExportRef::NamedReExport(named_re_export) => {
                export_bindings.push(named_re_export.get_binding_ast(ctx_ident, phase));
                export_props.extend(named_re_export.members.into_iter().map(
                    |member| match member {
                        ExportMember::Actual(actual_export) => {
                            actual_export.into_bind_prop(&named_re_export.mod_ident)
                        }
                        ExportMember::Binding(binding_export) => {
                            binding_export.into_bind_prop(&named_re_export.mod_ident)
                        }
                    },
                ));
            }
            ExportRef::ReExportAll(re_export_all) => {
                let ns_call =
                    to_ns_export(ctx_ident.clone(), re_export_all.mod_ident.clone().into());

                export_bindings.push(re_export_all.get_binding_ast(ctx_ident, phase));

                match re_export_all.name {
                    Some(exp_name) => export_props.push(kv_prop(exp_name, ns_call)),
                    None => export_props.insert(0, spread_prop(ns_call)),
                };
            }
        });

        let export_call = if export_props.len() > 0 {
            Some(exports_call(ctx_ident, export_props).into_stmt().into())
        } else {
            None
        };

        let export_value_decl = if export_value_bindings.len() > 0 {
            Some(
                VarDecl {
                    kind: VarDeclKind::Var,
                    decls: export_value_bindings,
                    ..Default::default()
                }
                .into(),
            )
        } else {
            None
        };

        (export_bindings, export_call, export_value_decl)
    }
}

pub struct ExportDeclItem {
    pub export_ref: ExportRef,
    pub export_stmt: ModuleItem,
    pub binding_stmt: ModuleItem,
}

pub struct ExportsAst {
    pub leading_body: Vec<ModuleItem>,
    pub trailing_body: Vec<ModuleItem>,
}

// NEW API
// Dependency of module
#[derive(Debug)]
pub struct Dep {
    pub src: String,
    pub members: Vec<DepMember>,
}

#[derive(Debug)]
pub struct DepMember {
    pub ident: Ident,
    pub name: Option<String>,
}

impl DepMember {
    pub fn new(ident: Ident, name: Option<String>) -> Self {
        DepMember { ident, name }
    }

    pub fn into_obj_pat_prop(self) -> ObjectPatProp {
        match self.name {
            Some(name) => ObjectPatProp::KeyValue(KeyValuePatProp {
                key: PropName::Ident(IdentName {
                    sym: name.into(),
                    span: DUMMY_SP,
                }),
                value: Box::new(Pat::Ident(self.ident.into())),
            }),
            None => ObjectPatProp::Assign(AssignPatProp {
                key: BindingIdent {
                    id: self.ident,
                    type_ann: None,
                },
                value: None,
                span: DUMMY_SP,
            }),
        }
    }
}

#[derive(Debug)]
pub enum Exp {
    Default(DefaultExp),
    ReExportAll(ReExportAllExp),
    ReExportNamed(ReExportNamedExp),
}

#[derive(Debug)]
pub struct DefaultExp {
    pub members: Vec<ExpMember>,
}

impl DefaultExp {
    pub fn new(members: Vec<ExpMember>) -> Self {
        Self { members }
    }

    pub fn into_exp_ast(self) -> (Vec<VarDeclarator>, Vec<PropOrSpread>, Vec<ExportSpecifier>) {
        let len = self.members.len();
        let mut declarators = Vec::with_capacity(len);
        let mut props = Vec::with_capacity(len);
        let mut specs = Vec::with_capacity(len);

        self.members.into_iter().for_each(|member| {
            declarators.push(VarDeclarator {
                name: Pat::Ident(member.ident.clone().into()),
                definite: false,
                init: None,
                span: DUMMY_SP,
            });

            props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(IdentName {
                    sym: member.name.clone().into(),
                    span: DUMMY_SP,
                }),
                value: Box::new(member.ident.clone().into()),
            }))));

            specs.push(ExportSpecifier::Named(ExportNamedSpecifier {
                orig: ModuleExportName::Ident(member.ident),
                exported: Some(ModuleExportName::Ident(Ident::from(member.name))),
                is_type_only: false,
                span: DUMMY_SP,
            }));
        });

        (declarators, props, specs)
    }
}

#[derive(Debug)]
pub struct ReExportAllExp {
    pub src: String,
    pub alias: Option<Ident>,
}

impl ReExportAllExp {
    pub fn default(src: String) -> Self {
        Self { src, alias: None }
    }

    pub fn alias(src: String, ident: Ident) -> Self {
        Self { src, alias: Some(ident) }
    }

    pub fn get_src(&self) -> String {
        self.src.clone()
    }

    pub fn to_import_stmt(&self, mod_ident: Ident) -> ModuleItem {
        ImportDecl {
            src: Box::new(self.get_src().into()),
            specifiers: vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
                local: mod_ident,
                span: DUMMY_SP,
            })],
            phase: ImportPhase::Evaluation,
            type_only: false,
            with: None,
            span: DUMMY_SP,
        }
        .into()
    }

    pub fn to_require_stmt(&self, ctx_ident: &Ident, mod_ident: Ident) -> Stmt {
        require_call(ctx_ident, self.get_src().clone().into())
            .into_var_decl(
                VarDeclKind::Const,
                Pat::Ident(BindingIdent {
                    id: mod_ident,
                    type_ann: None,
                }),
            )
            .into()
    }
    pub fn to_exp_props(&self, ctx_ident: &Ident, mod_ident: Ident) -> PropOrSpread {
        match &self.alias {
            Some(ident) => PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(IdentName {
                    sym: ident.sym.clone(),
                    span: DUMMY_SP,
                }),
                value: Box::new(mod_ident.into()),
            }))),
            None => PropOrSpread::Spread(SpreadElement {
                expr: Box::new(to_ns_export(ctx_ident.clone().into(), mod_ident.into()).into()),
                ..Default::default()
            }),
        }
    }
}

#[derive(Debug)]
pub struct ReExportNamedExp {
    pub src: String,
    pub members: Vec<ExpMember>,
}

impl ReExportNamedExp {
    fn get_src(&self) -> String {
        self.src.clone()
    }

    pub fn to_import_stmt(&self, mod_ident: Ident) -> ModuleItem {
        ImportDecl {
            src: Box::new(self.get_src().into()),
            specifiers: vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
                local: mod_ident,
                span: DUMMY_SP,
            })],
            phase: ImportPhase::Evaluation,
            type_only: false,
            with: None,
            span: DUMMY_SP,
        }
        .into()
    }

    pub fn to_require_stmt(&self, ctx_ident: &Ident, mod_ident: Ident) -> Stmt {
        require_call(ctx_ident, self.get_src().clone().into())
            .into_var_decl(
                VarDeclKind::Const,
                Pat::Ident(BindingIdent {
                    id: mod_ident,
                    type_ann: None,
                }),
            )
            .into()
    }

    pub fn to_exp_props(&self, mod_ident: Ident) -> Vec<PropOrSpread> {
        self.members
            .iter()
            .map(|member| {
                PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(IdentName {
                        sym: member.name.clone().into(),
                        span: DUMMY_SP,
                    }),
                    value: Box::new(
                        mod_ident
                            .clone()
                            .make_member(IdentName {
                                sym: member.ident.sym.clone(),
                                ..Default::default()
                            })
                            .into(),
                    ),
                })))
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct ExpMember {
    pub ident: Ident,
    pub name: String,
}

impl ExpMember {
    pub fn new(ident: Ident, name: String) -> Self {
        Self { ident, name }
    }
}

#[derive(Debug)]
pub struct ExpBinding {
    pub binding_ident: Ident,
    pub expr: Expr,
}

impl ExpBinding {
    pub fn to_assign_expr(self) -> Expr {
        assign_expr(self.binding_ident, self.expr).into()
    }
}
