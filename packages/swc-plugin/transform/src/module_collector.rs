use swc_core::{
    atoms::Atom,
    common::collections::AHashMap,
    ecma::{
        ast::*,
        utils::{private_ident, quote_ident},
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
};

use crate::{
    constants::{EXPORTS_ARG, REQUIRE_ARG},
    expr_utils::get_require_expr,
};

#[derive(Debug)]
pub enum ModuleRef {
    // `require('...');`
    Require,
    // `import ... from '...';`
    Import(Import),
    // `import('...');`
    DynImport(DynImport),
}

#[derive(Debug)]
pub struct Import {
    // `import def, { foo, bar as baz } from '...'`;
    // => def, foo, bar (alias: baz)
    pub members: Vec<ModuleMember>,
}

#[derive(Debug)]
pub struct DynImport {
    // `import('./foo');`
    // => Expr::Lit(Lit:Str)
    //
    // `import(expr);`
    // => Expr::Ident
    pub src_expr: Expr,
    // import('./foo', { with: ... });
    // => { with: ... }
    pub options: Option<Expr>,
}

impl DynImport {
    fn new(src_expr: &Expr, options: Option<Expr>) -> Self {
        DynImport {
            src_expr: src_expr.clone(),
            options,
        }
    }
}

#[derive(Debug)]
struct ModuleMember {
    // `import { foo } from 'foo'`;
    // => foo
    ident: Ident,
    // `import { foo as bar } from 'foo'`;
    // `import * as bar from 'foo'`;
    // => bar
    alias: Option<Ident>,
    // `import * as foo from 'foo';`
    // => true
    is_ns: bool,
}

impl ModuleMember {
    fn default(ident: &Ident, alias: Option<Ident>) -> Self {
        ModuleMember {
            ident: ident.clone(),
            alias,
            is_ns: false,
        }
    }

    fn ns(ident: &Ident) -> Self {
        ModuleMember {
            ident: ident.clone(),
            alias: None,
            is_ns: true,
        }
    }
}

pub struct ModuleCollector {
    // `import ... from './foo'`;
    // `require('./foo')`;
    //
    // key: './foo'
    // value: Dep
    pub mods: AHashMap<Atom, ModuleRef>,
    pub exports_ident: Ident,
    pub require_ident: Ident,
}

impl ModuleCollector {
    fn to_import_members(&self, specifiers: &Vec<ImportSpecifier>) -> Vec<ModuleMember> {
        let mut members = Vec::with_capacity(specifiers.len());

        specifiers.iter().for_each(|spec| match spec {
            ImportSpecifier::Default(ImportDefaultSpecifier { local, .. }) => {
                members.push(ModuleMember::default(
                    local,
                    Some(quote_ident!("default").into()),
                ));
            }
            ImportSpecifier::Named(ImportNamedSpecifier {
                local,
                imported,
                is_type_only: false,
                ..
            }) => {
                if let Some(ModuleExportName::Ident(ident)) = imported {
                    members.push(ModuleMember::default(ident, Some(local.clone())));
                } else {
                    members.push(ModuleMember::default(local, None));
                }
            }
            ImportSpecifier::Namespace(ImportStarAsSpecifier { local, .. }) => {
                members.push(ModuleMember::ns(local));
            }
            _ => {}
        });

        members
    }
}

impl Default for ModuleCollector {
    fn default() -> Self {
        ModuleCollector {
            mods: AHashMap::default(),
            exports_ident: private_ident!(EXPORTS_ARG),
            require_ident: private_ident!(REQUIRE_ARG),
        }
    }
}

impl VisitMut for ModuleCollector {
    noop_visit_mut_type!();

    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        for item in items.iter_mut() {
            match item {
                ModuleItem::Stmt(_) => item.visit_mut_children_with(self),
                ModuleItem::ModuleDecl(module_decl) => match module_decl {
                    ModuleDecl::Import(import) => {
                        let members = self.to_import_members(&import.specifiers);
                        let src = import.src.value.clone();

                        if let Some(ModuleRef::Import(mod_ref)) = self.mods.get_mut(&src) {
                            mod_ref.members.extend(members.into_iter());
                        } else {
                            self.mods.insert(
                                import.src.value.clone(),
                                ModuleRef::Import(Import { members }),
                            );
                        }
                    }
                    _ => { /* TODO */ }
                },
            }
        }
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Call(call_expr) => match call_expr {
                CallExpr {
                    args,
                    callee: Callee::Expr(callee_expr),
                    type_args: None,
                    ..
                } if args.len() == 1 && callee_expr.is_ident_ref_to("require") => {
                    let src = args.get(0).unwrap();

                    match &*src.expr {
                        Expr::Lit(Lit::Str(str)) => {
                            self.mods.insert(str.value.clone(), ModuleRef::Require);
                            *expr = get_require_expr(&self.require_ident, &src.expr);
                        }
                        _ => panic!("invalid `require()` call expression"),
                    }
                }
                CallExpr {
                    args,
                    callee: Callee::Import(_),
                    type_args: None,
                    ..
                } if args.len() >= 1 => {
                    let src = args.get(0).unwrap();
                    let options = args.get(1).and_then(|arg| Some(*arg.expr.clone()));

                    match &*src.expr {
                        Expr::Ident(Ident {
                            sym,
                            optional: false,
                            ..
                        }) => {
                            self.mods.insert(
                                sym.clone(),
                                ModuleRef::DynImport(DynImport::new(&*src.expr, options)),
                            );
                            *expr = get_require_expr(&self.require_ident, &src.expr);
                        }
                        Expr::Lit(Lit::Str(str)) => {
                            self.mods.insert(
                                str.value.clone(),
                                ModuleRef::DynImport(DynImport::new(&*src.expr, options)),
                            );
                            *expr = get_require_expr(&self.require_ident, &src.expr);
                        }
                        _ => {}
                    }
                }
                _ => expr.visit_mut_children_with(self),
            },
            _ => expr.visit_mut_children_with(self),
        }
    }
}
