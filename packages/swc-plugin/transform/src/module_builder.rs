use std::{iter, option::IntoIter};

use crate::{
    models::{Dep, Exp, LazyDep},
    module_collector::ModuleCollector,
    phase::ModulePhase,
    utils::ast::{
        arrow_with_paren_expr, mod_ident,
        presets::{
            define_call, exports_call, require_call, to_dep_getter_expr, to_deps_decl,
            to_empty_deps_decl, to_named_exps,
        },
        var_declarator,
    },
};
use swc_core::{
    common::{SyntaxContext, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{private_ident, ExprFactory},
    },
};

pub struct ModuleBuilder {
    ctx_ident: Ident,
    deps_ident: Ident,
    pub imports: Vec<ModuleItem>,
    pub exports: Vec<ModuleItem>,
    pub stmts: Vec<Stmt>,
    pub binding_stmt: Stmt,
    pub dep_getters: Vec<(String, Expr)>,
    pub exp_props: Vec<PropOrSpread>,
    pub exp_decls: Vec<VarDeclarator>,
    pub exp_specs: Vec<ExportSpecifier>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self {
            ctx_ident: private_ident!("__context"),
            deps_ident: private_ident!("__deps"),
            imports: Vec::new(),
            exports: Vec::new(),
            stmts: Vec::new(),
            binding_stmt: Stmt::default(),
            dep_getters: Vec::new(),
            exp_props: Vec::new(),
            exp_decls: Vec::new(),
            exp_specs: Vec::new(),
        }
    }

    pub fn collect_module_body(&mut self, collector: &mut ModuleCollector, body: Vec<ModuleItem>) {
        self.collect(collector);
        body.into_iter().for_each(|item| match item {
            ModuleItem::ModuleDecl(ModuleDecl::Import(_)) => self.imports.push(item),
            ModuleItem::ModuleDecl(_) => self.exports.push(item),
            ModuleItem::Stmt(stmt) if !stmt.is_empty() => self.stmts.push(stmt),
            _ => {}
        });
    }

    pub fn collect_script_body(&mut self, collector: &mut ModuleCollector, body: Vec<Stmt>) {
        self.collect(collector);
        self.stmts.extend(body);
    }

    fn collect(&mut self, collector: &mut ModuleCollector) {
        self.collect_deps(collector);
        self.collect_exps(collector);
        self.collect_bindings(collector);
    }

    fn collect_deps(&mut self, collector: &mut ModuleCollector) {
        collector.take_deps().into_iter().for_each(|dep| match dep {
            Dep::Default(default_dep) => {
                let require_props = default_dep
                    .members
                    .into_iter()
                    .map(|member| member.into_obj_pat_prop())
                    .collect::<Vec<ObjectPatProp>>();

                self.dep_getters
                    .push((default_dep.src.clone(), to_dep_getter_expr(&require_props)));

                self.stmts.push(
                    VarDecl {
                        kind: VarDeclKind::Const,
                        decls: vec![var_declarator(
                            Pat::Object(ObjectPat {
                                props: require_props,
                                optional: false,
                                type_ann: None,
                                span: DUMMY_SP,
                            }),
                            Some(Box::new(require_call(
                                collector.ctx_ident,
                                default_dep.src.into(),
                            ))),
                        )],
                        ..Default::default()
                    }
                    .into(),
                )
            }
            Dep::Lazy(LazyDep { src, expr }) => {
                self.dep_getters.push((src, arrow_with_paren_expr(expr)))
            }
        });
    }

    fn collect_exps(&mut self, collector: &mut ModuleCollector) {
        collector.take_exps().into_iter().for_each(|exp| match exp {
            Exp::Default(exp) => {
                let (decls, props, specs) = exp.into_asts();

                self.exp_decls.extend(decls);
                self.exp_props.extend(props);
                self.exp_specs.extend(specs);
            }
            Exp::ReExportNamed(re_export_named) => {
                let mod_ident = mod_ident();
                let src = re_export_named.src.clone();
                let imp_stmt = re_export_named.to_import_stmt(mod_ident.clone());
                let getter = (src, arrow_with_paren_expr(mod_ident.clone().into()));
                let req_stmt =
                    re_export_named.to_require_stmt(collector.ctx_ident, mod_ident.clone());
                let exp_prop = re_export_named.to_exp_props(mod_ident);

                self.imports.push(imp_stmt);
                self.dep_getters.push(getter);
                self.stmts.push(req_stmt);
                self.exp_props.extend(exp_prop);
            }
            Exp::ReExportAll(re_export_all) => {
                let mod_ident = mod_ident();
                let src = re_export_all.get_src();
                let imp_stmt = re_export_all.to_import_stmt(mod_ident.clone());
                let getter = (src, arrow_with_paren_expr(mod_ident.clone().into()));
                let req_stmt =
                    re_export_all.to_require_stmt(collector.ctx_ident, mod_ident.clone());
                let exp_prop = re_export_all.to_exp_props(collector.ctx_ident, mod_ident);

                self.imports.push(imp_stmt);
                self.dep_getters.push(getter);
                self.stmts.push(req_stmt);
                self.exp_props.push(exp_prop);
            }
        });
    }

    fn collect_bindings(&mut self, collector: &mut ModuleCollector) {
        self.binding_stmt = Expr::Seq(SeqExpr {
            exprs: collector
                .take_bindings()
                .into_iter()
                .map(|binding| Box::new(binding.to_assign_expr()))
                .collect::<Vec<Box<Expr>>>(),
            ..Default::default()
        })
        .into_stmt();
    }

    pub fn build_module(self, id: &String, phase: ModulePhase) -> Vec<ModuleItem> {
        let deps_decl = if phase == ModulePhase::Bundle {
            to_deps_decl(&self.deps_ident, self.dep_getters)
        } else {
            to_empty_deps_decl(&self.deps_ident)
        };

        let stmts = vec![
            deps_decl.into(),
            define_call(
                id,
                &self.ctx_ident,
                &self.deps_ident,
                self.stmts
                    .into_iter()
                    .chain(vec![
                        self.binding_stmt,
                        exports_call(&self.ctx_ident, self.exp_props).into_stmt(),
                    ])
                    .collect(),
            )
            .into_stmt(),
        ];

        let imports = if phase == ModulePhase::Bundle {
            self.imports
        } else {
            vec![]
        };

        let exports = if phase == ModulePhase::Bundle {
            self.exports
                .into_iter()
                .chain(
                    if self.exp_specs.len() > 0 {
                        Some(to_named_exps(self.exp_specs))
                    } else {
                        None
                    }
                    .into_iter(),
                )
                .collect()
        } else {
            vec![]
        };

        imports
            .into_iter()
            .chain(stmts.into_iter().map(|stmt| stmt.into()))
            .chain(exports)
            .chain(
                if self.exp_decls.len() > 0 {
                    Some(
                        Decl::Var(Box::new(VarDecl {
                            decls: self.exp_decls,
                            kind: VarDeclKind::Var,
                            declare: false,
                            span: DUMMY_SP,
                            ctxt: SyntaxContext::default(),
                        }))
                        .into(),
                    )
                } else {
                    None
                }
                .into_iter(),
            )
            .collect()
    }

    pub fn build_script(self, id: &String, phase: ModulePhase) -> Vec<Stmt> {
        let deps_decl = if phase == ModulePhase::Bundle {
            to_deps_decl(&self.deps_ident, self.dep_getters)
        } else {
            to_empty_deps_decl(&self.deps_ident)
        };

        let stmts = vec![
            deps_decl.into(),
            define_call(
                id,
                &self.ctx_ident,
                &self.deps_ident,
                self.stmts
                    .into_iter()
                    .chain(vec![
                        self.binding_stmt,
                        exports_call(&self.ctx_ident, self.exp_props).into_stmt(),
                    ])
                    .collect(),
            )
            .into_stmt(),
        ];

        stmts
            .into_iter()
            .chain(
                if self.exp_decls.len() > 0 {
                    Some(
                        Decl::Var(Box::new(VarDecl {
                            decls: self.exp_decls,
                            kind: VarDeclKind::Var,
                            declare: false,
                            span: DUMMY_SP,
                            ctxt: SyntaxContext::default(),
                        }))
                        .into(),
                    )
                } else {
                    None
                }
                .into_iter(),
            )
            .collect()
    }
}
