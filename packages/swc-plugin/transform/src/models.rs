use presets::decl_require_deps_stmt;
use swc_core::{
    atoms::Atom,
    ecma::{
        ast::*,
        utils::{private_ident, ExprFactory},
    },
};

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
            ModuleExportName::Str(_) => unimplemented!("TODO"),
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
    pub id: Option<f64>,
    /// Exported members.
    pub members: Vec<ExportMember>,
}

impl NamedReExportRef {
    pub fn new(src: Atom, id: Option<f64>, members: Vec<ExportMember>) -> Self {
        NamedReExportRef {
            mod_ident: private_ident!("__mod"),
            src,
            id,
            members,
        }
    }

    pub fn get_binding_ast(&self, ctx_ident: Ident, phase: ModulePhase) -> ModuleItem {
        match phase {
            ModulePhase::Bundle => import_star(self.mod_ident.clone(), self.src.clone()),
            ModulePhase::Runtime => decl_require_deps_stmt(
                ctx_ident,
                self.id
                    .map(|id| Lit::from(id))
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
    pub id: Option<f64>,
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
    pub fn new(src: Atom, id: Option<f64>, name: Option<Atom>) -> Self {
        ReExportAllRef {
            mod_ident: private_ident!("__mod"),
            src,
            id,
            name,
        }
    }

    pub fn get_binding_ast(&self, ctx_ident: Ident, phase: ModulePhase) -> ModuleItem {
        match phase {
            ModulePhase::Bundle => import_star(self.mod_ident.clone(), self.src.clone()),
            ModulePhase::Runtime => decl_require_deps_stmt(
                ctx_ident,
                self.id
                    .map(|id| Lit::from(id))
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
        deps_id: &Option<AHashMap<String, f64>>,
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
                        id.copied(),
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
                        id.copied(),
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

    /// Converts `ExportDecl` into `ExportDeclItem`.
    pub fn get_from_export_decl(export_decl: &ExportDecl) -> ExportDeclItem {
        // `function foo {}`
        //
        // - `orig_ident`: `foo`
        // - `decl_expr`: `function foo{}`
        let (orig_ident, decl_expr) = get_expr_from_decl(&export_decl.decl);

        // - `binding_name`: `__x`
        // - `exported_name`: `foo`
        let binding_export = BindingExportMember::new(orig_ident.sym.clone());
        let binding_name = ModuleExportName::Ident(binding_export.bind_ident.clone());
        let exported_name = ModuleExportName::Ident(orig_ident);

        // `binding_assign_stmt`: `__x = function foo {}`
        let binding_assign_stmt =
            assign_expr(binding_export.bind_ident.clone(), decl_expr).into_stmt();
        let export_ref = ExportRef::Named(NamedExportRef::new(vec![ExportMember::Binding(
            binding_export,
        )]));

        ExportDeclItem {
            export_ref,
            export_stmt: ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
                specifiers: vec![ExportSpecifier::Named(ExportNamedSpecifier {
                    orig: binding_name,
                    exported: exported_name.into(),
                    is_type_only: false,
                    span: DUMMY_SP,
                })],
                src: None,
                with: None,
                type_only: false,
                span: DUMMY_SP,
            }))
            .into(),
            binding_stmt: binding_assign_stmt.into(),
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
    /// var { default: React, useState, useCallback } = ctx_ident.require('react');
    /// var { core } = ctx_ident.require('@app/core');
    /// var ns = ctx_ident.require('@app/internal');
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
                    decl_require_deps_stmt(ctx_ident.clone(), src.clone(), ident.clone().into())
                        .into(),
                ),
            });

        if dep_props.len() > 0 {
            requires.push(
                decl_require_deps_stmt(
                    ctx_ident.clone(),
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
        deps_id: &Option<AHashMap<String, f64>>,
    ) -> Vec<ModuleItem> {
        let mut items = vec![];

        deps.iter().for_each(|(src, value)| {
            let src_lit = if let Some(deps_id) = deps_id {
                if let Some(id) = deps_id.get(src.as_str()) {
                    Lit::from(*id).into()
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

    /// Converts exports into `ExportsAst`.
    pub fn exports_to_ast(
        ctx_ident: &Ident,
        exports: Vec<ExportRef>,
        phase: ModulePhase,
    ) -> ExportsAst {
        let mut leading_body: Vec<ModuleItem> = vec![];
        let mut export_props: Vec<PropOrSpread> = vec![];
        let mut export_decls: Vec<VarDeclarator> = vec![];

        exports.into_iter().for_each(|export_ref| match export_ref {
            ExportRef::Named(NamedExportRef { members }) => {
                members.into_iter().for_each(|member| match member {
                    ExportMember::Actual(actual_export) => {
                        export_props.push(actual_export.into());
                    }
                    ExportMember::Binding(binding_export) => {
                        export_props.push(binding_export.clone().into());
                        export_decls.push(binding_export.into());
                    }
                })
            }
            ExportRef::NamedReExport(named_re_export) => {
                leading_body.push(named_re_export.get_binding_ast(ctx_ident.clone(), phase));
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

                leading_body.push(re_export_all.get_binding_ast(ctx_ident.clone(), phase));

                match re_export_all.name {
                    Some(exp_name) => export_props.push(kv_prop(exp_name, ns_call)),
                    None => export_props.insert(0, spread_prop(ns_call)),
                };
            }
        });

        let mut trailing_body = vec![];
        trailing_body.push(
            exports_call(ctx_ident.clone(), obj_lit_expr(export_props))
                .into_stmt()
                .into(),
        );

        if export_decls.len() > 0 {
            trailing_body.push(
                VarDecl {
                    kind: VarDeclKind::Var,
                    decls: export_decls,
                    ..Default::default()
                }
                .into(),
            )
        }

        ExportsAst {
            leading_body,
            trailing_body,
        }
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
