use swc_core::{
    atoms::Atom,
    ecma::{ast::*, utils::private_ident},
};

#[derive(Debug)]
pub enum ModuleRef {
    // `require('...');`
    Require(RequireRef),
    // `import ... from '...';`
    Import(ImportRef),
    // `import('...');`
    DynImport(DynImportRef),
}

#[derive(Debug)]
pub struct RequireRef {
    pub orig_expr: Expr,
}

impl RequireRef {
    pub fn new(orig_expr: &Expr) -> Self {
        RequireRef {
            orig_expr: orig_expr.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ImportRef {
    // `import def, { foo, bar as baz } from '...'`;
    // => def, foo, bar (alias: baz)
    pub members: Vec<ImportMember>,
}

impl ImportRef {
    pub fn new(members: Vec<ImportMember>) -> Self {
        ImportRef { members }
    }
}

#[derive(Debug)]
pub struct DynImportRef {
    pub orig_expr: Expr,
}

impl DynImportRef {
    pub fn new(orig_expr: &Expr) -> Self {
        DynImportRef {
            orig_expr: orig_expr.clone(),
        }
    }
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
    /// A reference variable that points to the export target.
    ///
    /// ```js
    /// __x = actualExportMember;
    /// // => __x
    /// ```
    pub export_ident: Ident,
    /// A reference identifier for the import statement used in re-export handling.
    ///
    /// ```js
    /// import * as __rx from './foo';
    /// // => __rx
    /// ```
    pub mod_ident: Ident,
    /// Alias identifier.
    ///
    /// ```js
    /// export * as foo from './foo';
    /// // => Some(foo)
    ///
    /// export * from './foo';
    /// // => None
    /// ```
    pub ident: Option<Ident>,
}

impl ImportNamespaceMember {
    pub fn alias(ident: &Ident) -> Self {
        ImportNamespaceMember {
            export_ident: private_ident!("__x"),
            mod_ident: private_ident!("__rx"),
            ident: Some(ident.clone()),
        }
    }

    pub fn anonymous() -> Self {
        ImportNamespaceMember {
            export_ident: private_ident!("__x"),
            mod_ident: private_ident!("__rx"),
            ident: None,
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
pub struct ExportMember {
    /// A reference variable that points to the export target.
    ///
    /// ```js
    /// __x = actualExportMember;
    /// // => __x
    /// ```
    pub export_ident: Ident,
    /// The identifier of the actual export target.
    ///
    /// ```js
    /// __x = actualExportMember;
    /// // => actualExportMember
    /// ```
    pub orig_ident: Option<Ident>,
    /// Export name.
    pub name: Atom,
}

impl ExportMember {
    pub fn new(orig_ident: &Ident, name: Option<Atom>) -> Self {
        let exp_name = if let Some(name) = name {
            name
        } else {
            orig_ident.sym.clone()
        };

        ExportMember {
            export_ident: private_ident!("__x"),
            orig_ident: Some(orig_ident.clone()),
            name: exp_name,
        }
    }

    pub fn anonymous(name: Atom) -> Self {
        ExportMember {
            export_ident: private_ident!("__x"),
            orig_ident: None,
            name,
        }
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
    /// A reference variable that points to the export target.
    ///
    /// ```js
    /// __x = actualExportMember;
    /// // => __x
    /// ```
    pub exp_ident: Ident,
    /// Source of the referenced module.
    pub src: Atom,
    /// Exported members.
    pub members: Vec<ExportMember>,
}

impl NamedReExportRef {
    pub fn new(
        mod_ident: &Ident,
        exp_ident: &Ident,
        src: &Atom,
        members: Vec<ExportMember>,
    ) -> Self {
        NamedReExportRef {
            mod_ident: mod_ident.clone(),
            exp_ident: exp_ident.clone(),
            src: src.clone(),
            members,
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
    /// A reference variable that points to the export target.
    ///
    /// ```js
    /// __x = actualExportMember;
    /// // => __x
    /// ```
    pub exp_ident: Ident,
    /// Source of the referenced module.
    pub src: Atom,
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
    pub fn new(mod_ident: &Ident, exp_ident: &Ident, src: &Atom, name: Option<Atom>) -> Self {
        ReExportAllRef {
            mod_ident: mod_ident.clone(),
            exp_ident: exp_ident.clone(),
            src: src.clone(),
            name,
        }
    }
}
