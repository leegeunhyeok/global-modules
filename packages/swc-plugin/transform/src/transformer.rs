use core::panic;
use std::mem;

use crate::{
    models::{Exp, ReExportExp},
    module_collector::ModuleCollector,
    phase::ModulePhase,
    utils::ast::{
        mod_ident,
        presets::{define_call, exports_call, require_call, to_dep_getter_expr, to_named_exps},
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
    id: String,
    ctx_ident: Ident,
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
        let deps = collector.take_deps();
        let exps = collector.take_exps();

        let deps_count = deps.len();
        let mut require_calls = Vec::with_capacity(deps_count);
        let mut dep_getters = Vec::with_capacity(deps_count);

        let mut imports: Vec<ModuleItem> = Vec::new();
        let mut exports = Vec::new();
        let mut stmts = Vec::new();

        let mut exp_props: Vec<PropOrSpread> = Vec::new();
        let mut exp_decls: Vec<VarDeclarator> = Vec::new();
        let mut exp_specs: Vec<ExportSpecifier> = Vec::new();

        deps.into_iter().for_each(|dep| {
            let require_props = dep
                .members
                .into_iter()
                .map(|member| member.into_obj_pat_prop())
                .collect::<Vec<ObjectPatProp>>();

            dep_getters.push(to_dep_getter_expr(&require_props));
            stmts.push(
                VarDecl {
                    kind: VarDeclKind::Const,
                    decls: vec![VarDeclarator {
                        name: Pat::Object(ObjectPat {
                            props: require_props,
                            optional: false,
                            type_ann: None,
                            span: DUMMY_SP,
                        }),
                        definite: false,
                        init: Some(Box::new(require_call(&self.ctx_ident, dep.src.into()))),
                        span: DUMMY_SP,
                    }],
                    ..Default::default()
                }
                .into(),
            );
        });

        exps.into_iter().for_each(|exp| match exp {
            Exp::Default(exp) => {
                let (declarators, props, specs) = exp.into_exp_ast();

                exp_decls.extend(declarators);
                exp_props.extend(props);
                exp_specs.extend(specs);
            }
            Exp::ReExport(re_export) => {
                let mod_ident = mod_ident();

                imports.push(re_export.to_import_stmt(mod_ident.clone()));
                require_calls.push(re_export.to_require_stmt(&self.ctx_ident, mod_ident.clone()));
                exp_props.extend(re_export.to_exp_props(&self.ctx_ident, mod_ident));
            }
        });

        body.into_iter().for_each(|item| match item {
            ModuleItem::ModuleDecl(ref module_decl) => match module_decl {
                ModuleDecl::Import(_) => imports.push(item),
                _ => exports.push(item),
            },
            ModuleItem::Stmt(stmt) => stmts.push(stmt),
        });

        stmts.push(exports_call(&self.ctx_ident, exp_props).into_stmt());

        if exp_specs.len() > 0 {
            exports.push(to_named_exps(exp_specs));
        }

        let mut body = [
            imports,
            vec![define_call(&self.id, &self.ctx_ident, stmts)
                .into_stmt()
                .into()],
            exports,
        ]
        .concat();

        if exp_decls.len() > 0 {
            // var __x, __x1, ...;
            body.push(
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

        module.body = body;
    }
}
