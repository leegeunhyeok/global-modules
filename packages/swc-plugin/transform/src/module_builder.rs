use std::collections::hash_map::Entry;

use crate::{
    models::{Dep, DepGetter, Exp, RuntimeDep},
    module_collector::ModuleCollector,
    utils::ast::*,
    utils::presets::*,
};
use swc_core::{
    common::{collections::AHashMap, SyntaxContext, DUMMY_SP},
    ecma::{ast::*, utils::ExprFactory},
};

pub struct ModuleBuilder<'a> {
    /// Context identifier
    ctx_ident: &'a Ident,
    /// Dependencies identifier
    deps_ident: &'a Ident,
    /// Imports statements
    imports: Vec<ModuleItem>,
    /// Exports statements
    exports: Vec<ModuleItem>,
    /// Statements
    stmts: Vec<Stmt>,
    /// Binding statement
    ///
    /// ```js
    /// // Actual module statements
    /// // <- Binding statement
    /// // Exports call expression
    /// ```
    binding_stmt: Stmt,
    /// Dependency getters
    ///
    /// ```js
    /// const __deps = {
    ///   "dep_1": () => expr,
    ///   "dep_2": () => expr,
    /// };
    /// ```
    pub dep_getters: AHashMap<String, DepGetter>,
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
    pub fn new(ctx_ident: &'a Ident, deps_ident: &'a Ident) -> Self {
        Self {
            ctx_ident,
            deps_ident,
            imports: Vec::new(),
            exports: Vec::new(),
            stmts: Vec::new(),
            binding_stmt: Stmt::default(),
            dep_getters: AHashMap::default(),
            exp_props: Vec::new(),
            exp_decls: Vec::new(),
            exp_specs: Vec::new(),
        }
    }

    /// Collects ASTs from the collector and module body.
    pub fn collect_module_body(&mut self, collector: &mut ModuleCollector, body: Vec<ModuleItem>) {
        self.collect(collector);
        body.into_iter().for_each(|item| match item {
            ModuleItem::ModuleDecl(ModuleDecl::Import(_)) => self.imports.push(item),
            ModuleItem::ModuleDecl(_) => self.exports.push(item),
            ModuleItem::Stmt(stmt) if !stmt.is_empty() => self.stmts.push(stmt),
            _ => {}
        });
    }

    /// Collects ASTs from the collector and script body.
    pub fn collect_script_body(&mut self, collector: &mut ModuleCollector, body: Vec<Stmt>) {
        self.collect(collector);
        self.stmts.extend(body);
    }

    /// Appends a dependency property to the dependency properties map.
    fn insert_dep_getter(&mut self, src: String, getter: DepGetter) {
        match self.dep_getters.entry(src) {
            Entry::Occupied(mut entry) => match (entry.get_mut(), getter) {
                (DepGetter::Props(prev_props), DepGetter::Props(new_props)) => {
                    prev_props.extend(new_props);
                }
                (DepGetter::Expr(prev_expr), DepGetter::Expr(expr)) => {
                    *prev_expr = expr;
                }
                _ => unreachable!(),
            },
            Entry::Vacant(entry) => drop(entry.insert(getter)),
        };
    }

    /// Collects ASTs from the collected dependencies, exports, and bindings
    fn collect(&mut self, collector: &mut ModuleCollector) {
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

                self.insert_dep_getter(src.clone(), DepGetter::Props(require_props.clone()));
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
                            Some(Box::new(require_call(collector.ctx_ident, src.into()))),
                        )],
                        ..Default::default()
                    }
                    .into(),
                )
            }
            Dep::Runtime(RuntimeDep { src, expr }) => {
                self.insert_dep_getter(src, DepGetter::Expr(arrow_with_paren_expr(expr)));
            }
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
                let getter_expr = arrow_with_paren_expr(mod_ident.clone().into());
                let req_stmt =
                    to_require_stmt(&collector.ctx_ident, mod_ident.clone(), src.clone());
                let exp_prop = re_export_named.to_exp_props(mod_ident);

                self.imports.push(imp_stmt);
                self.stmts.push(req_stmt);
                self.exp_props.extend(exp_prop);
                self.insert_dep_getter(src, DepGetter::Expr(getter_expr));
            }
            Exp::ReExportAll(re_export_all) => {
                let mod_ident = mod_ident();
                let src = re_export_all.src.clone();
                let imp_stmt = to_import_all_stmt(mod_ident.clone(), src.clone());
                let getter_expr = arrow_with_paren_expr(mod_ident.clone().into());
                let req_stmt =
                    to_require_stmt(&collector.ctx_ident, mod_ident.clone(), src.clone());
                let exp_prop = re_export_all.to_exp_props(collector.ctx_ident, mod_ident);

                self.imports.push(imp_stmt);
                self.stmts.push(req_stmt);
                self.exp_props.push(exp_prop);
                self.insert_dep_getter(src, DepGetter::Expr(getter_expr));
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

    /// Returns a list of statements that can be used to source type: 'module'
    pub fn build_module(self, id: &String, runtime: bool) -> Vec<ModuleItem> {
        let deps_decl = if runtime {
            to_empty_deps_decl(&self.deps_ident)
        } else {
            to_deps_decl(&self.deps_ident, self.dep_getters)
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

        let imports = if runtime { vec![] } else { self.imports };
        let exports = if runtime {
            vec![]
        } else {
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

    /// Returns a list of statements that can be used to source type: 'script'
    pub fn build_script(self, id: &String, runtime: bool) -> Vec<Stmt> {
        let deps_decl = if runtime {
            to_empty_deps_decl(&self.deps_ident)
        } else {
            to_deps_decl(&self.deps_ident, self.dep_getters)
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
