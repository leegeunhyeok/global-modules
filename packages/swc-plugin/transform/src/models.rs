use presets::require_call;
use swc_core::{
    atoms::Atom,
    common::{Spanned, DUMMY_SP},
    ecma::{ast::*, utils::ExprFactory},
    plugin::errors::HANDLER,
};

use crate::utils::ast::*;

#[derive(Debug)]
pub struct ModuleRef {
    // `import def, { foo, bar as baz } from '...'`;
    // => def, foo, bar (alias: baz)
    pub members: Vec<ImportMember>,
}

#[derive(Debug)]
pub enum ImportMember {
    Default(ImportDefaultMember),
    Named(ImportNamedMember),
    Namespace(ImportNamespaceMember),
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
            Some(name) => obj_kv_prop(name.into(), self.ident.into()),
            None => obj_assign_prop(self.ident.into()),
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
            declarators.push(var_declarator(member.ident.clone().into(), None));
            props.push(kv_prop(
                member.name.clone().into(),
                member.ident.clone().into(),
            ));
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
        Self {
            src,
            alias: Some(ident),
        }
    }

    pub fn get_src(&self) -> String {
        self.src.clone()
    }

    pub fn to_import_stmt(&self, mod_ident: Ident) -> ModuleItem {
        import_star(mod_ident, self.get_src().into())
    }

    pub fn to_require_stmt(&self, ctx_ident: &Ident, mod_ident: Ident) -> Stmt {
        require_call(ctx_ident, self.get_src().clone().into())
            .into_var_decl(VarDeclKind::Const, mod_ident.into())
            .into()
    }
    pub fn to_exp_props(&self, ctx_ident: &Ident, mod_ident: Ident) -> PropOrSpread {
        match &self.alias {
            Some(ident) => kv_prop(ident.sym.clone(), mod_ident.into()),
            None => spread_prop(to_ns_export(ctx_ident.clone().into(), mod_ident.into())),
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
            .into_var_decl(VarDeclKind::Const, mod_ident.into())
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
