use swc_core::ecma::{ast::*, utils::quote_ident};

use crate::models::*;

/* ----- ImportMember ----- */

impl From<&ImportDefaultSpecifier> for ImportMember {
    fn from(value: &ImportDefaultSpecifier) -> Self {
        ImportMember::Named(ImportNamedMember::new(
            &quote_ident!("default").into(),
            Some(value.local.clone()),
        ))
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
        ImportMember::Namespace(ImportNamespaceMember::alias(&value.local))
    }
}

/* ----- ExportMember ----- */

impl From<&ExportNamedSpecifier> for ExportMember {
    fn from(value: &ExportNamedSpecifier) -> Self {
        match &value.orig {
            ModuleExportName::Ident(orig_ident) => ExportMember::new(
                &orig_ident,
                Some(
                    if let Some(ModuleExportName::Ident(exported_ident)) = &value.exported {
                        exported_ident
                    } else {
                        orig_ident
                    }
                    .sym
                    .clone(),
                ),
            ),
            ModuleExportName::Str(_) => unimplemented!("TODO"),
        }
    }
}
