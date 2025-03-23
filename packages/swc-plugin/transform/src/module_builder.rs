use std::iter;

use crate::{
    models::{Dep, Exp},
    module_collector::ModuleCollector,
    utils::ast::*,
    utils::presets::*,
};
use swc_core::{
    common::{SyntaxContext, DUMMY_SP},
    ecma::{ast::*, utils::ExprFactory},
};

pub struct ModuleBuilder<'a> {
    /// Context identifier
    ctx_ident: &'a Ident,
    /// Imports statements for re-exports bindings
    bind_imports: Vec<ModuleItem>,
    /// global module's `require` call statements
    req_calls: Vec<Stmt>,
    /// Binding statement
    ///
    /// ```js
    /// // Actual module statements
    /// // <- Binding statement
    /// // Exports call expression
    /// ```
    binding_stmt: Option<Stmt>,
    /// Export properties
    ///
    /// ```js
    /// // Vector of `PropOrSpread`
    /// // key: foo, value: __x
    /// // key: bar, value: __x1
    /// // key: baz, value: __x2
    ///
    /// // Will be transformed into
    /// context.exports({
    ///   "foo": __x,
    ///   "bar": __x1,
    ///   "baz": __x2,
    /// });
    /// ```
    pub exp_props: Vec<PropOrSpread>,
    /// Export var declarators
    ///
    /// ```js
    /// // Vector of `VarDeclarator`
    /// // "foo", "bar", "baz"
    ///
    /// // Will be transformed into
    /// var foo, bar, baz;
    /// ```
    pub exp_decls: Vec<VarDeclarator>,
    /// Export specifiers
    ///
    /// ```js
    /// // Vector of `ExportSpecifier`
    /// // name: "foo", ident: __x
    /// // name: "bar", ident: __x1
    /// // name: "default", ident: __x2
    ///
    /// // Will be transformed into
    /// export { foo as __x, bar as __x1, default as __x2 };
    /// ```
    pub exp_specs: Vec<ExportSpecifier>,
}

impl<'a> ModuleBuilder<'a> {
    pub fn new(ctx_ident: &'a Ident) -> Self {
        Self {
            ctx_ident,
            bind_imports: Vec::new(),
            req_calls: Vec::new(),
            binding_stmt: None,
            exp_props: Vec::new(),
            exp_decls: Vec::new(),
            exp_specs: Vec::new(),
        }
    }

    /// Collects ASTs from the collected dependencies, exports, and bindings
    pub fn collect(&mut self, collector: &mut ModuleCollector) {
        self.collect_deps(collector);
        self.collect_exps(collector);
        self.collect_bindings(collector);
    }

    /// Collects ASTs from the collected dependencies
    fn collect_deps(&mut self, collector: &mut ModuleCollector) {
        collector.take_deps().into_iter().for_each(|dep| match dep {
            Dep::Base(base_dep) => {
                let src = base_dep.src;
                let require_props = base_dep
                    .members
                    .into_iter()
                    .map(|member| member.into_obj_pat_prop())
                    .collect::<Vec<ObjectPatProp>>();

                self.req_calls.push(
                    VarDecl {
                        kind: VarDeclKind::Const,
                        decls: vec![var_declarator(
                            Pat::Object(ObjectPat {
                                props: require_props,
                                optional: false,
                                type_ann: None,
                                span: DUMMY_SP,
                            }),
                            Some(Box::new(require_call(src.into()))),
                        )],
                        ..Default::default()
                    }
                    .into(),
                )
            }
            _ => {}
        });
    }

    /// Collects ASTs from the collected exports
    fn collect_exps(&mut self, collector: &mut ModuleCollector) {
        collector.take_exps().into_iter().for_each(|exp| match exp {
            Exp::Base(exp) => {
                let (decls, props, specs) = exp.into_asts();

                self.exp_decls.extend(decls);
                self.exp_props.extend(props);
                self.exp_specs.extend(specs);
            }
            Exp::ReExportNamed(re_export_named) => {
                let mod_ident = mod_ident();
                let src = re_export_named.src.clone();
                let imp_stmt = to_import_namespace_stmt(mod_ident.clone(), src.clone());
                let req_stmt = to_require_stmt(mod_ident.clone(), src.clone());
                let exp_prop = re_export_named.to_exp_props(mod_ident);

                self.bind_imports.push(imp_stmt);
                self.req_calls.push(req_stmt);
                self.exp_props.extend(exp_prop);
            }
            Exp::ReExportAll(re_export_all) => {
                let mod_ident = mod_ident();
                let src = re_export_all.src.clone();
                let imp_stmt = to_import_all_stmt(mod_ident.clone(), src.clone());
                let req_stmt = to_require_stmt(mod_ident.clone(), src.clone());
                let exp_prop = re_export_all.to_exp_props(collector.ctx_ident, mod_ident);

                self.bind_imports.push(imp_stmt);
                self.req_calls.push(req_stmt);
                self.exp_props.push(exp_prop);
            }
        });
    }

    /// Collects bindings from the collector and
    /// creates a statement that assigns them to the each binding
    ///
    /// ```js
    /// // binding_stmt
    /// __x = foo, __x1 = bar, __x2 = baz;
    /// ```
    fn collect_bindings(&mut self, collector: &mut ModuleCollector) {
        let bindings = collector.take_bindings();

        if bindings.is_empty() {
            return;
        }

        self.binding_stmt = Some(
            Expr::Seq(SeqExpr {
                exprs: bindings
                    .into_iter()
                    .map(|binding| Box::new(binding.to_assign_expr()))
                    .collect::<Vec<Box<Expr>>>(),
                ..Default::default()
            })
            .into_stmt(),
        );
    }

    /// Returns a list of statements that can be used to source type: 'module'
    pub fn build_module(
        self,
        id: &String,
        runtime: bool,
        orig_module: Vec<ModuleItem>,
    ) -> Vec<ModuleItem> {
        let exports_call = if self.exp_props.is_empty() {
            None
        } else {
            Some(exports_call(&self.ctx_ident, self.exp_props).into_stmt())
        };

        let exp_var_decl = if self.exp_decls.len() > 0 {
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
        };

        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut stmts = vec![];

        let context_decl = register_call(id).into_var_decl(
            VarDeclKind::Const,
            Pat::Ident(self.ctx_ident.clone().into()),
        );

        orig_module.into_iter().for_each(|item| match item {
            ModuleItem::ModuleDecl(ref module_decl) => match module_decl {
                ModuleDecl::Import(_) => imports.push(item),
                _ => exports.push(item),
            },
            ModuleItem::Stmt(ref stmt) if !matches!(stmt, Stmt::Empty(_)) => stmts.push(item),
            _ => {}
        });

        let extra_stmts = vec![self.binding_stmt, exports_call, exp_var_decl]
            .into_iter()
            .filter_map(|stmt| match stmt {
                Some(stmt) if !matches!(stmt, Stmt::Empty(_)) => Some(stmt.into()),
                _ => None,
            });

        if runtime {
            vec![context_decl.into()]
                .into_iter()
                .chain(self.req_calls)
                .map(|stmt| stmt.into())
                .chain(stmts)
                .chain(extra_stmts)
                .collect::<Vec<ModuleItem>>()
        } else {
            imports
                .into_iter()
                .chain(self.bind_imports)
                .chain(iter::once(context_decl.into()))
                .chain(stmts)
                .chain(extra_stmts)
                .chain(
                    exports.into_iter().chain(
                        if self.exp_specs.len() > 0 {
                            Some(to_named_exps(self.exp_specs))
                        } else {
                            None
                        }
                        .into_iter(),
                    ),
                )
                .collect::<Vec<ModuleItem>>()
        }
    }

    /// Returns a list of statements that can be used to source type: 'script'
    pub fn build_script(self, id: &String, orig_script: Vec<Stmt>) -> Vec<Stmt> {
        let exports_call = if self.exp_props.is_empty() {
            None
        } else {
            Some(exports_call(&self.ctx_ident, self.exp_props).into_stmt())
        };

        let exp_var_decl = if self.exp_decls.len() > 0 {
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
        };

        let context_decl = register_call(id).into_var_decl(
            VarDeclKind::Const,
            Pat::Ident(self.ctx_ident.clone().into()),
        );

        vec![context_decl.into()]
            .into_iter()
            .chain(self.req_calls)
            .chain(orig_script)
            .chain(self.binding_stmt.into_iter())
            .chain(exports_call.into_iter())
            .chain(exp_var_decl.into_iter())
            .collect()
    }
}
