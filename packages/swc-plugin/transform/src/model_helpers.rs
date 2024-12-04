use swc_core::ecma::ast::*;

use crate::models::*;

/* ----- ImportMember ----- */

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

/* ----- ExportMember ----- */

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
