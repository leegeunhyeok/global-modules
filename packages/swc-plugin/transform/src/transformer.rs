use core::panic;
use std::{iter, mem};

use crate::{
    models::{Dep, Exp, LazyDep},
    module_collector::ModuleCollector,
    phase::ModulePhase,
    utils::ast::{
        arrow_with_paren_expr, mod_ident,
        presets::{
            define_call, exports_call, require_call, to_dep_getter_expr, to_deps_decl,
            to_named_exps,
        },
        var_declarator,
    },
};
use swc_core::{
    common::{collections::AHashMap, SyntaxContext, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{private_ident, ExprFactory},
        visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
    },
};

pub struct GlobalModuleTransformer {
    id: String,
    ctx_ident: Ident,
    deps_ident: Ident,
    phase: ModulePhase,
    unresolved_ctxt: SyntaxContext,
    paths: Option<AHashMap<String, String>>,
}

impl GlobalModuleTransformer {
    pub fn new(
        id: String,
        phase: ModulePhase,
        paths: Option<AHashMap<String, String>>,
        unresolved_ctxt: SyntaxContext,
    ) -> Self {
        Self {
            id,
            phase,
            unresolved_ctxt,
            paths,
            ctx_ident: private_ident!("__context"),
            deps_ident: private_ident!("__deps"),
        }
    }
}

impl VisitMut for GlobalModuleTransformer {
    noop_visit_mut_type!();

    fn visit_mut_script(&mut self, script: &mut Script) {
        let mut collector =
            ModuleCollector::new(self.unresolved_ctxt, &self.ctx_ident, &self.paths);

        script.visit_mut_children_with(&mut collector);

        // Replace to new script body.
        // script.body = self.get_script_body(mem::take(&mut script.body));

        panic!("TODO");
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        let mut collector =
            ModuleCollector::new(self.unresolved_ctxt, &self.ctx_ident, &self.paths);

        module.visit_mut_children_with(&mut collector);

        let body = mem::take(&mut module.body);
        let mut builder = ModuleBuilder::new(&mut collector);

        body.into_iter().for_each(|item| match item {
            ModuleItem::ModuleDecl(ModuleDecl::Import(_)) => builder.imports.push(item),
            ModuleItem::ModuleDecl(_) => builder.exports.push(item),
            ModuleItem::Stmt(stmt) if !stmt.is_empty() => builder.stmts.push(stmt),
            _ => {}
        });

        module.body = builder.build(&self.id, &self.ctx_ident, &self.deps_ident);
    }
}

struct ModuleBuilder {
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
    pub fn new(collector: &mut ModuleCollector) -> Self {
        let mut builder = Self {
            imports: Vec::new(),
            exports: Vec::new(),
            stmts: Vec::new(),
            binding_stmt: Stmt::default(),
            dep_getters: Vec::new(),
            exp_props: Vec::new(),
            exp_decls: Vec::new(),
            exp_specs: Vec::new(),
        };

        builder.collect_deps(collector);
        builder.collect_exps(collector);
        builder.collect_bindings(collector);
        builder
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

    pub fn build(self, id: &String, ctx_ident: &Ident, deps_ident: &Ident) -> Vec<ModuleItem> {
        let has_exp_specs = self.exp_specs.len() > 0;
        let has_exp_decls = self.exp_decls.len() > 0;
        let stmts = self
            .stmts
            .into_iter()
            .chain(vec![
                self.binding_stmt,
                exports_call(ctx_ident, self.exp_props).into_stmt(),
            ])
            .collect();

        self.imports
            .into_iter()
            .chain(iter::once(
                to_deps_decl(deps_ident, self.dep_getters).into(),
            ))
            .chain(iter::once(
                define_call(id, ctx_ident, deps_ident, stmts)
                    .into_stmt()
                    .into(),
            ))
            .chain(self.exports)
            .chain(
                if has_exp_specs {
                    Some(to_named_exps(self.exp_specs))
                } else {
                    None
                }
                .into_iter(),
            )
            .chain(
                if has_exp_decls {
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
