use swc_core::{
    common::DUMMY_SP,
    ecma::{ast::*, utils::ExprFactory},
};

use crate::utils::ast::*;

// Dependency
#[derive(Debug)]
pub enum Dep {
    /// Dependency
    ///
    /// ```js
    /// import '...';
    /// ```
    Base(BaseDep),
    /// Runtime dependency
    ///
    /// ```js
    /// // CommonJS
    /// require('...');
    ///
    /// // ESModule
    /// import(...);
    /// ```
    Runtime,
}

impl Dep {
    /// Creates a new default dependency
    pub fn base(src: String, members: Vec<DepMember>) -> Self {
        Dep::Base(BaseDep { src, members })
    }

    /// Creates a new runtime dependency
    pub fn runtime() -> Self {
        Dep::Runtime
    }
}

#[derive(Debug)]
pub struct BaseDep {
    /// Source
    pub src: String,
    /// Members
    pub members: Vec<DepMember>,
}

#[derive(Debug)]
pub struct DepMember {
    /// Identifier
    pub ident: Ident,
    /// Name
    pub name: Option<String>,
}

impl DepMember {
    /// Creates a new dependency member
    pub fn new(ident: Ident, name: Option<String>) -> Self {
        DepMember { ident, name }
    }

    /// Converts to an object property
    ///
    /// ```js
    /// { foo: foo, bar: bar, default: baz }
    /// ```
    pub fn into_obj_pat_prop(self) -> ObjectPatProp {
        match self.name {
            Some(name) => obj_kv_prop(name.into(), self.ident.into()),
            None => obj_assign_prop(self.ident.into()),
        }
    }
}

#[derive(Debug)]
pub enum Exp {
    /// Base export
    ///
    /// ```js
    /// export { foo, bar as baz };
    /// ```
    Base(BaseExp),
    /// Re-export all
    ///
    /// ```js
    /// export * from '...';
    /// ```
    ReExportAll(ReExportAllExp),
    /// Re-export named
    ///
    /// ```js
    /// export { foo, bar as baz } from '...';
    /// export * as foo from '...';
    /// ```
    ReExportNamed(ReExportNamedExp),
}
#[derive(Debug)]
pub struct BaseExp {
    /// Export members
    pub members: Vec<ExpMember>,
}

impl BaseExp {
    /// Creates a new base export
    pub fn new(members: Vec<ExpMember>) -> Self {
        Self { members }
    }

    /// Converts to ASTs
    ///
    /// - `Vec<VarDeclarator>`
    /// - `Vec<PropOrSpread>`
    /// - `Vec<ExportSpecifier>`
    pub fn into_asts(self) -> (Vec<VarDeclarator>, Vec<PropOrSpread>, Vec<ExportSpecifier>) {
        let len = self.members.len();

        // To declare binding variables
        //
        // ```js
        // var __x, __x1, __x2;
        // ```
        let mut declarators = Vec::with_capacity(len);

        // To declare export properties
        //
        // ```js
        // context.exports({
        //   "foo": __x,
        //   "bar": __x1,
        //   "baz": __x2,
        // });
        // ```
        let mut props = Vec::with_capacity(len);

        // To declare export specifiers
        //
        // ```js
        // export { foo as __x, bar as __x1, default as __x2 };
        // ```
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
    /// Source
    pub src: String,
    /// Alias
    pub alias: Option<Ident>,
}

impl ReExportAllExp {
    /// Creates a new re-export all
    pub fn new(src: String) -> Self {
        Self { src, alias: None }
    }

    /// Creates a new re-export all with alias
    pub fn alias(src: String, ident: Ident) -> Self {
        Self {
            src,
            alias: Some(ident),
        }
    }

    /// Converts to export properties
    ///
    /// ```js
    /// {
    ///   // Re-export with alias
    ///   "foo": ctx_ident.exports.ns(mod_ident),
    ///   // Re-export all
    ///   ...ctx_ident.exports.ns(mod_ident),
    /// }
    /// ```
    pub fn to_exp_props(&self, ctx_ident: &Ident, mod_ident: Ident) -> PropOrSpread {
        match &self.alias {
            Some(ident) => kv_prop(
                ident.sym.clone(),
                to_ns_export(ctx_ident.clone().into(), mod_ident.into()),
            ),
            None => spread_prop(to_ns_export(ctx_ident.clone().into(), mod_ident.into())),
        }
    }
}

#[derive(Debug)]
pub struct ReExportNamedExp {
    /// Source
    pub src: String,
    /// Members
    pub members: Vec<ExpMember>,
}

impl ReExportNamedExp {
    /// Converts to export properties
    ///
    /// ```js
    /// {
    ///   "foo": mod_ident,
    ///   "bar": mod_ident,
    /// }
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
    /// Identifier
    pub ident: Ident,
    /// Name
    pub name: String,
}

impl ExpMember {
    /// Creates a new export member
    pub fn new(ident: Ident, name: String) -> Self {
        Self { ident, name }
    }
}

/// Binding data for references the module exports.
///
/// ```js
/// // Code
/// export const foo = 1;
/// export default bar;
/// export * from './baz';
/// ```
///
/// `ExpBinding` is data struct for the transform to following code:
///
/// ```js
/// // Declaration of binding identifiers
/// var __x, __x1, __x2;
///
/// // Binding export members into binding identifiers
/// import * as __mod from './baz';
/// __x = foo;
/// __x1 = bar;
/// __x2 = __mod;
///
/// // Collected `ExpBinding`:
/// // { binding_ident: __x, expr: foo }
/// // { binding_ident: __x1, expr: bar }
/// // { binding_ident: __x2, expr: __mod }
/// ```
#[derive(Debug)]
pub struct ExpBinding {
    /// Binding identifier
    pub binding_ident: Ident,
    /// Expression
    pub expr: Expr,
}

impl ExpBinding {
    /// Converts to assignment expression
    ///
    /// ```js
    /// binding_ident = expr;
    /// ```
    pub fn to_assign_expr(self) -> Expr {
        assign_expr(self.binding_ident, self.expr).into()
    }
}
