use swc_core::{
    atoms::Atom,
    ecma::{ast::*, utils::private_ident},
};

use crate::constants::EXPORTS;

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
pub struct ImportMember {
    // `import { foo } from 'foo';`
    // `import * as foo from 'foo';`
    // => foo
    pub ident: Ident,
    // `import { foo as bar } from 'foo'`;
    // => bar
    pub alias: Option<Ident>,
    // `true` if name spaced import.
    pub is_ns: bool,
}

impl ImportMember {
    pub fn default(ident: &Ident, alias: Option<Ident>) -> Self {
        ImportMember {
            ident: ident.clone(),
            alias,
            is_ns: false,
        }
    }

    pub fn ns(ident: &Ident) -> Self {
        ImportMember {
            ident: ident.clone(),
            alias: None,
            is_ns: true,
        }
    }
}

#[derive(Debug)]
pub enum ExportRef {
    Named(NamedExportRef),
    NamedReExport(NamedReExportRef),
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
    pub exp_ident: Ident,
    pub orig_ident: Ident,
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
            exp_ident: private_ident!(EXPORTS),
            orig_ident: orig_ident.clone(),
            name: exp_name,
        }
    }

    pub fn anonymous(name: Atom) -> Self {
        let ident = private_ident!(EXPORTS);

        ExportMember {
            exp_ident: ident.clone(),
            orig_ident: ident,
            name,
        }
    }
}

#[derive(Debug)]
pub struct NamedReExportRef {
    pub ident: Ident,
    pub src: Atom,
    pub members: Vec<ExportMember>,
}

#[derive(Debug)]
pub struct ReExportAllRef {
    pub ident: Ident,
    pub src: Atom,
    pub name: Option<Atom>,
}
