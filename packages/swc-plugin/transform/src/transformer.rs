use core::panic;
use std::mem;

use crate::{
    module_collector::ModuleCollector,
    phase::ModulePhase,
    utils::ast::{
        mod_ident,
        presets::{define_call, exports_call, require_call},
        to_ns_export,
    },
};
use swc_core::{
    common::{collections::AHashMap, SyntaxContext, DUMMY_SP},
    ecma::{
        ast::*,
        transforms::base::quote,
        utils::{private_ident, quote_ident, ExprFactory},
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
};
use tracing::debug;

pub struct GlobalModuleTransformer {
    collector: ModuleCollector,
    id: String,
    phase: ModulePhase,
    unresolved_ctxt: SyntaxContext,
}

impl GlobalModuleTransformer {
    pub fn new(
        id: String,
        phase: ModulePhase,
        paths: Option<AHashMap<String, String>>,
        unresolved_ctxt: SyntaxContext,
    ) -> Self {
        Self {
            collector: ModuleCollector::new(unresolved_ctxt, paths),
            id,
            phase,
            unresolved_ctxt,
        }
    }
}

impl VisitMut for GlobalModuleTransformer {
    noop_visit_mut_type!();

    fn visit_mut_script(&mut self, script: &mut Script) {
        script.visit_mut_children_with(&mut self.collector);

        // Replace to new script body.
        // script.body = self.get_script_body(mem::take(&mut script.body));

        panic!("TODO");
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(&mut self.collector);

        debug!("deps: {:?}", self.collector.deps);
        debug!("exps: {:?}", self.collector.exps);

        let body = mem::take(&mut module.body);
        let deps = mem::take(&mut self.collector.deps);
        let exps = mem::take(&mut self.collector.exps);

        let mut mod_imports: Vec<ModuleItem> = Vec::new();
        let mut require_deps: Vec<ModuleItem> = deps
            .into_iter()
            .map(|dep| {
                let props = dep
                    .members
                    .into_iter()
                    .map(|member| {
                        if member.name.is_some() {
                            ObjectPatProp::KeyValue(KeyValuePatProp {
                                key: PropName::Ident(IdentName {
                                    sym: member.name.unwrap().into(),
                                    span: DUMMY_SP,
                                }),
                                value: Box::new(Pat::Ident(member.ident.into())),
                            })
                        } else {
                            ObjectPatProp::Assign(AssignPatProp {
                                key: BindingIdent {
                                    id: member.ident,
                                    type_ann: None,
                                },
                                value: None,
                                span: DUMMY_SP,
                            })
                        }
                    })
                    .collect::<Vec<ObjectPatProp>>();

                let var_decl = VarDecl {
                    kind: VarDeclKind::Const,
                    decls: vec![VarDeclarator {
                        name: Pat::Object(ObjectPat {
                            props,
                            optional: false,
                            type_ann: None,
                            span: DUMMY_SP,
                        }),
                        definite: false,
                        init: Some(Box::new(require_call(dep.src.into()))),
                        span: DUMMY_SP,
                    }],
                    ..Default::default()
                };

                ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(var_decl))))
            })
            .collect::<Vec<ModuleItem>>();

        let mut exp_props: Vec<PropOrSpread> = Vec::new();
        let mut exp_decls: Vec<VarDeclarator> = Vec::new();
        let mut exp_specs: Vec<ExportSpecifier> = Vec::new();
        exps.into_iter().for_each(|exp| {
            if exp.src.is_some() {
                let src = exp.src.unwrap();
                let mod_ident = mod_ident();
                let var_decl = VarDecl {
                    kind: VarDeclKind::Const,
                    decls: vec![VarDeclarator {
                        name: Pat::Ident(BindingIdent {
                            id: mod_ident.clone(),
                            type_ann: None,
                        }),
                        definite: false,
                        init: Some(Box::new(require_call(src.clone().into()))),
                        span: DUMMY_SP,
                    }],
                    ..Default::default()
                };

                require_deps.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(var_decl)))));
                mod_imports.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    src: Box::new(src.into()),
                    specifiers: vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
                        local: mod_ident.clone(),
                        span: DUMMY_SP,
                    })],
                    phase: ImportPhase::Evaluation,
                    type_only: false,
                    with: None,
                    span: DUMMY_SP,
                })));

                if exp.members.len() == 0 {
                    // export all
                    exp_props.push(PropOrSpread::Spread(SpreadElement {
                        expr: Box::new(
                            to_ns_export(quote_ident!("ctx").into(), mod_ident.into()).into(),
                        ),
                        ..Default::default()
                    }));
                } else {
                    debug!("exp: {:?}", exp.members);

                    exp.members.into_iter().for_each(|member| {
                        exp_props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(
                            KeyValueProp {
                                key: PropName::Ident(IdentName {
                                    sym: member.name.clone().into(),
                                    span: DUMMY_SP,
                                }),
                                value: Box::new(if member.is_ns {
                                    mod_ident.clone().into()
                                } else {
                                    mod_ident
                                        .clone()
                                        .make_member(IdentName {
                                            sym: member.ident.sym,
                                            ..Default::default()
                                        })
                                        .into()
                                }),
                            },
                        ))));
                    });
                }
            } else {
                exp.members.into_iter().for_each(|member| {
                    exp_decls.push(VarDeclarator {
                        name: Pat::Ident(member.ident.clone().into()),
                        definite: false,
                        init: None,
                        span: DUMMY_SP,
                    });

                    exp_props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Ident(IdentName {
                            sym: member.name.clone().into(),
                            span: DUMMY_SP,
                        }),
                        value: Box::new(member.ident.clone().into()),
                    }))));

                    exp_specs.push(ExportSpecifier::Named(ExportNamedSpecifier {
                        orig: ModuleExportName::Ident(member.ident),
                        exported: Some(ModuleExportName::Ident(Ident::from(member.name))),
                        is_type_only: false,
                        span: DUMMY_SP,
                    }));
                });
            }
        });

        // exps
        let exports_call = exports_call(
            quote_ident!("ctx").into(),
            ObjectLit {
                props: exp_props,
                ..Default::default()
            }
            .into(),
        );

        let mut new_body = [
            mod_imports,
            require_deps,
            body,
            vec![exports_call.into_stmt().into()],
        ]
        .concat();

        if exp_specs.len() > 0 {
            new_body.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
                NamedExport {
                    specifiers: exp_specs,
                    type_only: false,
                    src: None,
                    with: None,
                    span: DUMMY_SP,
                },
            )))
        }

        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut stmts = Vec::new();
        new_body.into_iter().for_each(|item| match item {
            ModuleItem::ModuleDecl(ref module_decl) => match module_decl {
                ModuleDecl::Import(_) => imports.push(item),
                _ => exports.push(item),
            },
            ModuleItem::Stmt(stmt) => stmts.push(stmt),
        });

        if exp_decls.len() > 0 {
            // TODO: append decls without exports variable (this is actually not an export)
            // var __x, __x1, ...;
            exports.push(
                Decl::Var(Box::new(VarDecl {
                    decls: exp_decls,
                    kind: VarDeclKind::Var,
                    declare: false,
                    span: DUMMY_SP,
                    ctxt: SyntaxContext::default(),
                }))
                .into(),
            );
        }

        module.body = [
            imports,
            vec![define_call(&self.id, quote_ident!("ctx").into(), stmts)
                .into_stmt()
                .into()],
            exports,
        ]
        .concat();
    }
}
