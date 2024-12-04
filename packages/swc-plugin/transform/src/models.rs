use swc_core::{
    atoms::Atom,
    ecma::{ast::*, utils::private_ident},
};

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

    pub fn anonymous() -> Self {
        Self {
            ident: private_ident!("__mod"),
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

#[derive(Debug)]
pub struct ActualExportMember {
    /// The identifier of the actual export target.
    ///
    /// ```js
    /// __x = actualExportMember;
    /// // => actualExportMember
    /// ```
    pub ident: Ident,
    /// Export name.
    pub name: Atom,
}

impl ActualExportMember {
    pub fn new(orig_ident: &Ident, name: Option<Atom>) -> Self {
        let name = if let Some(name) = name {
            name
        } else {
            orig_ident.sym.clone()
        };

        Self {
            ident: orig_ident.clone(),
            name,
        }
    }
}

#[derive(Debug)]
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
    /// Exported members.
    pub members: Vec<ExportMember>,
}

impl NamedReExportRef {
    pub fn new(mod_ident: &Ident, src: &Atom, members: Vec<ExportMember>) -> Self {
        NamedReExportRef {
            mod_ident: mod_ident.clone(),
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
    pub fn new(mod_ident: &Ident, src: &Atom, name: Option<Atom>) -> Self {
        ReExportAllRef {
            mod_ident: mod_ident.clone(),
            src: src.clone(),
            name,
        }
    }
}
