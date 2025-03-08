pub mod ast {
    use crate::models::*;
    use core::panic;
    use std::mem;
    use swc_core::{
        atoms::Atom,
        common::{collections::AHashMap, Spanned, SyntaxContext, DUMMY_SP},
        ecma::{
            ast::*,
            utils::{private_ident, ExprFactory},
        },
        plugin::errors::HANDLER,
    };
    use tracing::{debug, field::debug};

    pub fn anonymous_default_binding_ident() -> Ident {
        private_ident!("__default")
    }

    pub fn exp_binding_ident() -> Ident {
        private_ident!("__x")
    }

    pub fn mod_ident() -> Ident {
        private_ident!("__mod")
    }

    /// Returns a key-value property ast.
    ///
    /// ```js
    /// // Code
    /// { key: value }
    ///
    /// // Property
    /// // => key: value
    /// ```
    pub fn kv_prop(key: Atom, value: Expr) -> PropOrSpread {
        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: PropName::Str(Str {
                value: key,
                raw: None,
                span: DUMMY_SP,
            }),
            value: value.into(),
        })))
    }

    /// Returns a spread property ast.
    ///
    /// ```js
    /// // Code
    /// { ...expr };
    ///
    /// // Property
    /// // => ...expr
    /// ```
    pub fn spread_prop(expr: Expr) -> PropOrSpread {
        PropOrSpread::Spread(SpreadElement {
            expr: expr.into(),
            ..Default::default()
        })
    }

    /// Returns an assign expression.
    ///
    /// ```js
    /// // Code
    /// ident = expr;
    /// ```
    pub fn assign_expr(ident: Ident, expr: Expr) -> AssignExpr {
        AssignExpr {
            left: AssignTarget::Simple(SimpleAssignTarget::Ident(ident.into())),
            right: expr.into(),
            op: AssignOp::Assign,
            ..Default::default()
        }
    }

    /// Returns an object literal expression.
    ///
    /// ```js
    /// // Code
    /// { prop: value, prop_1: value }
    /// ```
    pub fn obj_lit_expr(props: Vec<PropOrSpread>) -> Expr {
        Expr::Object(ObjectLit {
            props,
            ..Default::default()
        })
    }

    /// Returns a string literal expression.
    ///
    /// ```js
    /// // Code
    /// 'foo'
    /// ```
    pub fn str_lit_expr(str: &String) -> Expr {
        Expr::Lit(Lit::Str(Str {
            value: str.as_str().into(),
            raw: None,
            span: DUMMY_SP,
        }))
    }

    /// Returns a variable declarator bound to the provided identifier.
    ///
    /// ```js
    /// // Code
    /// var foo, bar, baz;
    ///
    /// // VarDeclarators
    /// // => foo, bar, baz
    /// ```
    pub fn var_declarator(ident: Ident) -> VarDeclarator {
        VarDeclarator {
            name: Pat::Ident(ident.into()),
            definite: false,
            init: None,
            span: DUMMY_SP,
        }
    }

    /// Returns an import-all statement bound to the provided identifier.
    ///
    /// ```js
    /// // Code
    /// import * as ident from 'src';
    /// ```
    pub fn import_star(ident: Ident, src: Atom) -> ModuleItem {
        ModuleDecl::Import(ImportDecl {
            phase: ImportPhase::Evaluation,
            specifiers: vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
                local: ident,
                span: DUMMY_SP,
            })],
            src: Box::new(src.into()),
            type_only: false,
            with: None,
            span: DUMMY_SP,
        })
        .into()
    }

    /// Wraps the given expression with a `__ctx.exports.ns` function call expression.
    ///
    /// ```js
    /// // Code
    /// __ctx.exports.ns(<expr>);
    /// ```
    pub fn to_ns_export(ident: Ident, expr: Expr) -> Expr {
        ident
            .make_member("exports".into())
            .make_member("ns".into())
            .as_call(DUMMY_SP, vec![expr.into()])
    }

    /// Checks whether it is a member expression of a CommonJS module.
    ///
    /// ```js
    /// // Code
    /// exports.foo; // true;
    /// ```
    pub fn is_cjs_exports_member(member_expr: &MemberExpr, unresolved_ctxt: SyntaxContext) -> bool {
        member_expr.obj.is_ident_ref_to("exports")
            && member_expr.obj.as_ident().unwrap().ctxt == unresolved_ctxt
    }

    /// Checks whether it is a member expression of a CommonJS module.
    ///
    /// ```js
    /// // Code
    /// module.exports; // true;
    /// ```
    pub fn is_cjs_module_member(member_expr: &MemberExpr, unresolved_ctxt: SyntaxContext) -> bool {
        member_expr.obj.is_ident_ref_to("module")
            && member_expr.obj.as_ident().unwrap().ctxt == unresolved_ctxt
            && member_expr.prop.is_ident_with("exports")
    }

    /// Returns a new expression that assigns to a member expression.
    ///
    /// ```js
    /// // left = right;
    /// member.prop = right_expr;
    /// ```
    pub fn assign_member(left: MemberExpr, right: Expr) -> Expr {
        right.make_assign_to(
            AssignOp::Assign,
            AssignTarget::Simple(SimpleAssignTarget::Member(left)),
        )
    }

    pub fn arrow_with_paren_expr(expr: Expr) -> Expr {
        Expr::Arrow(ArrowExpr {
            body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::Paren(ParenExpr {
                span: DUMMY_SP,
                expr: Box::new(expr),
            })))),
            ..Default::default()
        })
    }

    /// Returns a new expression that binds the CommonJS module export statement.
    ///
    ///
    pub fn get_new_assign_expr(
        ctx_ident: Ident,
        expr: Expr,
        export_name: Option<Expr>,
    ) -> Option<Expr> {
        // `ctx_ident.module;`
        let ctx_module = ctx_ident
            .make_member(IdentName {
                sym: "module".into(),
                ..Default::default()
            })
            .make_member(IdentName {
                sym: "exports".into(),
                ..Default::default()
            });

        assign_member(
            match export_name {
                Some(name_expr) => match name_expr {
                    Expr::Lit(Lit::Str(str_lit)) => ctx_module.make_member(IdentName {
                        sym: str_lit.value.clone().into(),
                        ..Default::default()
                    }),
                    _ => ctx_module.computed_member(name_expr.clone()),
                },
                None => ctx_module,
            },
            expr,
        )
        .into()
    }

    /// Returns a new expression that binds the CommonJS module export statement.
    ///
    /// ```js
    /// // Input
    /// module.exports = orig_expr;
    ///
    /// // ModulePhase::Bundle
    /// ctx_ident.module.exports = module.exports = orig_expr;
    /// ctx_ident.module.exports.foo = module.exports.foo = orig_expr;
    ///
    /// // ModulePhase::Runtime
    /// ctx_ident.module.exports = orig_expr;
    /// ctx_ident.module.exports.foo = orig_expr;
    /// ```
    pub fn to_binding_module_from_assign_expr(
        ctx_ident: Ident,
        assign_expr: &AssignExpr,
        unresolved_ctxt: SyntaxContext,
    ) -> Option<Expr> {
        if assign_expr.op != AssignOp::Assign {
            return None;
        }

        match &assign_expr.left {
            AssignTarget::Simple(SimpleAssignTarget::Member(member_expr)) => {
                if is_cjs_exports_member(member_expr, unresolved_ctxt) {
                    // `exports.foo = ...;`
                    // `exports['foo'] = ...;`
                    get_new_assign_expr(
                        ctx_ident,
                        *assign_expr.right.clone(),
                        get_expr_from_member_prop(&member_expr.prop).into(),
                    )
                } else if is_cjs_module_member(member_expr, unresolved_ctxt) {
                    // `module.exports = ...;`
                    get_new_assign_expr(ctx_ident, *assign_expr.right.clone(), None)
                } else if let Some(leading_member) = member_expr.obj.as_member() {
                    if is_cjs_module_member(leading_member, unresolved_ctxt) {
                        // `module.exports.foo = ...;`
                        // `module.exports['foo'] = ...;`
                        get_new_assign_expr(
                            ctx_ident,
                            *assign_expr.right.clone(),
                            get_expr_from_member_prop(&member_expr.prop).into(),
                        )
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Returns a new expression that binds the CommonJS module export statement.
    ///
    /// ```js
    /// // Input
    /// module.exports; // Object.assign(module.exports, { ... });
    ///
    /// // ModulePhase::Bundle
    /// ctx_ident.module.exports = module.exports;
    /// ctx_ident.module.exports.foo = module.exports.foo;
    ///
    /// // ModulePhase::Runtime
    /// ctx_ident.module.exports;
    /// ctx_ident.module.exports.foo;
    /// ```
    pub fn to_binding_module_from_member_expr(
        ctx_ident: Ident,
        member_expr: &MemberExpr,
        unresolved_ctxt: SyntaxContext,
    ) -> Option<MemberExpr> {
        if is_cjs_module_member(member_expr, unresolved_ctxt) == false {
            return None;
        }

        let ctx_module_member = ctx_ident
            .make_member(IdentName {
                sym: "module".into(),
                ..Default::default()
            })
            .make_member(IdentName {
                sym: "exports".into(),
                ..Default::default()
            });

        ctx_module_member.into()
    }

    /// Extracts and returns the its ident from the declarations.
    ///
    /// ```js
    /// function foo {}
    /// class Bar {}
    /// const baz = expr;
    /// ```
    pub fn get_ident_from_decl(decl: &Decl) -> Option<Ident> {
        match decl {
            Decl::Class(ClassDecl {
                ident,
                declare: false,
                ..
            }) => Some(ident.clone()),
            Decl::Fn(FnDecl {
                ident,
                declare: false,
                ..
            }) => Some(ident.clone()),
            Decl::Var(val_decl) => {
                if val_decl.decls.len() != 1 {
                    HANDLER.with(|handler| {
                        handler
                            .struct_span_err(
                                val_decl.span,
                                "multiple variable declarations are not allowed",
                            )
                            .emit();
                    });
                    panic!(); // FIXME
                }

                let var_decl = val_decl.decls.get(0).unwrap();

                match var_decl {
                    VarDeclarator {
                        name: Pat::Ident(BindingIdent { id, type_ann: None }),
                        definite: false,
                        ..
                    } => Some(id.clone()),
                    _ => {
                        HANDLER.with(|handler| {
                            handler
                                .struct_span_err(var_decl.span, "unsupported variable declaration")
                                .emit();
                        });
                        panic!(); // FIXME
                    }
                }
            }
            _ => None,
        }
    }

    /// Extracts and returns the its ident from the declarations.
    ///
    /// ```js
    /// export default function foo() {};
    /// export default class Bar {}
    /// ```
    pub fn get_ident_from_default_decl(default_decl: &DefaultDecl) -> Option<Ident> {
        match default_decl {
            DefaultDecl::Class(ClassExpr { ident, .. }) => ident.clone().into(),
            DefaultDecl::Fn(FnExpr { ident, .. }) => ident.clone().into(),
            _ => None,
        }
    }

    /// Extracts and returns the expression from the default declaration statement.
    ///
    /// ```js
    /// // Input
    /// export default function foo() {};
    /// export default class Bar {};
    ///
    /// // Code
    /// function foo() {};
    /// class Bar {};
    /// ```
    pub fn get_expr_from_default_decl(default_decl: &DefaultDecl) -> Expr {
        match default_decl {
            DefaultDecl::Class(class_expr) => Expr::Class(class_expr.clone()),
            DefaultDecl::Fn(fn_expr) => Expr::Fn(fn_expr.clone()),
            _ => HANDLER.with(|handler| {
                handler
                    .struct_span_err(
                        default_decl.span(),
                        "unsupported default export declaration",
                    )
                    .emit();

                Expr::default()
            }),
        }
    }

    /// Converts `DefaultDecl` into `Decl`.
    pub fn into_decl(default_decl: &DefaultDecl) -> Decl {
        match default_decl {
            DefaultDecl::Class(class_expr) => class_expr.clone().as_class_decl().unwrap().into(),
            DefaultDecl::Fn(fn_expr) => fn_expr.clone().as_fn_decl().unwrap().into(),
            _ => HANDLER.with(|handler| {
                handler
                    .struct_span_err(
                        default_decl.span(),
                        "unsupported default export declaration",
                    )
                    .emit();

                Decl::Var(Box::new(VarDecl {
                    kind: VarDeclKind::Var,
                    decls: vec![],
                    ..Default::default()
                }))
            }),
        }
    }

    pub fn get_src_lit(lit: &Lit, deps_id: &Option<AHashMap<String, String>>) -> Lit {
        match lit {
            Lit::Str(str_lit) => {
                let src = str_lit.value.clone();
                let src_lit = deps_id
                    .as_ref()
                    .and_then(|deps_id| deps_id.get(src.as_str()))
                    .map_or_else(|| str_lit.clone().into(), |id| Lit::from(id.as_str()));

                src_lit
            }
            _ => HANDLER.with(|handler| {
                handler
                    .struct_span_err(lit.span(), "unsupported literal type")
                    .emit();

                Lit::Null(Null { span: DUMMY_SP })
            }),
        }
    }

    pub fn get_expr_from_member_prop(prop: &MemberProp) -> Expr {
        match prop {
            MemberProp::Ident(ident) => Expr::Lit(Lit::Str(Str {
                value: ident.sym.as_str().into(),
                raw: None,
                span: DUMMY_SP,
            })),
            MemberProp::Computed(ComputedPropName { expr, .. }) => match &**expr {
                str_lit_expr @ Expr::Lit(Lit::Str(_)) => str_lit_expr.clone(),
                _ => *expr.clone(),
            },
            _ => HANDLER.with(|handler| {
                handler
                    .struct_span_err(prop.span(), "unsupported property type")
                    .emit();

                panic!("fatal error");
            }),
        }
    }

    // NEW API
    pub fn import_as_dep(import_decl: &ImportDecl) -> Option<Dep> {
        let src = import_decl.src.value.to_string();
        let members = import_decl
            .specifiers
            .iter()
            .filter_map(|spec| match spec {
                ImportSpecifier::Named(ImportNamedSpecifier {
                    imported,
                    local,
                    is_type_only: false,
                    ..
                }) => Some(DepMember::default(
                    local.clone(),
                    imported.as_ref().map(|name| match name {
                        ModuleExportName::Ident(ident) => ident.sym.as_str().to_string(),
                        ModuleExportName::Str(str) => str.value.to_string(),
                    }),
                )),
                ImportSpecifier::Default(ImportDefaultSpecifier { local, .. }) => {
                    Some(DepMember::default(local.clone(), Some("default".into())))
                }
                ImportSpecifier::Namespace(ImportStarAsSpecifier { local, .. }) => {
                    Some(DepMember::ns(local.clone(), None))
                }
                _ => None,
            })
            .collect::<Vec<DepMember>>();

        if members.is_empty() {
            None
        } else {
            Dep { src, members }.into()
        }
    }

    pub fn export_decl_as_exp(export_decl: &ExportDecl) -> Option<(Exp, Stmt, ExpBinding)> {
        if let Some(decl_ident) = get_ident_from_decl(&export_decl.decl) {
            let exp_binding_ident = exp_binding_ident();
            let name = decl_ident.sym.as_str().to_string();
            let exp = Exp::Default(DefaultExp::new(vec![ExpMember::new(
                exp_binding_ident.clone(),
                name,
            )]));

            Some((
                exp,
                Stmt::Decl(export_decl.decl.clone()),
                ExpBinding {
                    binding_ident: exp_binding_ident,
                    expr: decl_ident.into(),
                },
            ))
        } else {
            None
        }
    }

    pub fn export_default_decl_as_exp(
        export_default_decl: &ExportDefaultDecl,
    ) -> Option<(Exp, Decl, ExpBinding)> {
        if let Some(decl) = match &export_default_decl.decl {
            DefaultDecl::Class(class_expr) => {
                let class_ident = class_expr
                    .ident
                    .clone()
                    .unwrap_or_else(|| anonymous_default_binding_ident());

                Some(Decl::Class(ClassDecl {
                    ident: class_ident.clone(),
                    class: class_expr.class.clone(),
                    declare: false,
                }))
            }
            DefaultDecl::Fn(fn_expr) => {
                let fn_ident = fn_expr
                    .ident
                    .clone()
                    .unwrap_or_else(|| anonymous_default_binding_ident());

                Some(Decl::Fn(FnDecl {
                    ident: fn_ident,
                    function: fn_expr.function.clone(),
                    declare: false,
                }))
            }
            DefaultDecl::TsInterfaceDecl(_) => None,
        } {
            let binding_ident = match &decl {
                Decl::Class(class_decl) => class_decl.ident.clone(),
                Decl::Fn(fn_decl) => fn_decl.ident.clone(),
                _ => unreachable!(),
            };
            let exp_binding_ident = exp_binding_ident();
            let exp = Exp::Default(DefaultExp::new(vec![ExpMember::new(
                exp_binding_ident.clone(),
                "default".into(),
            )]));

            Some((
                exp,
                decl,
                ExpBinding {
                    binding_ident: exp_binding_ident,
                    expr: binding_ident.into(),
                },
            ))
        } else {
            None
        }
    }

    pub fn export_default_expr_as_exp(
        export_default_expr: &mut ExportDefaultExpr,
    ) -> (Exp, Stmt, ExpBinding) {
        let binding_ident = anonymous_default_binding_ident();
        let exp_binding_ident = exp_binding_ident();
        let exp = Exp::Default(DefaultExp::new(vec![ExpMember::new(
            exp_binding_ident.clone(),
            "default".into(),
        )]));

        let default_var_decl = VarDecl {
            decls: vec![VarDeclarator {
                name: binding_ident.clone().into(),
                init: Some(Box::new(*export_default_expr.expr.clone())),
                definite: false,
                span: DUMMY_SP,
            }],
            kind: VarDeclKind::Const,
            ..Default::default()
        };

        (
            exp,
            default_var_decl.into(),
            ExpBinding {
                binding_ident: exp_binding_ident,
                expr: binding_ident.into(),
            },
        )
    }

    pub fn export_named_as_exp(export_named: &NamedExport) -> Option<(Exp, Vec<ExpBinding>)> {
        let mut exp_bindings: Vec<ExpBinding> = Vec::new();
        let members = export_named
            .specifiers
            .iter()
            .filter_map(|spec| match spec {
                ExportSpecifier::Default(default) => {
                    let exp_binding_ident = exp_binding_ident();

                    exp_bindings.push(ExpBinding {
                        binding_ident: exp_binding_ident.clone(),
                        expr: Expr::from(default.exported.clone()),
                    });

                    Some(ExpMember::new(exp_binding_ident, "default".into()))
                }
                ExportSpecifier::Named(ExportNamedSpecifier {
                    orig,
                    exported,
                    is_type_only: false,
                    ..
                }) => {
                    let exp_binding_ident = exp_binding_ident();
                    let exported_ident = match orig {
                        ModuleExportName::Ident(ident) => ident.clone(),
                        ModuleExportName::Str(str) => {
                            Ident::new(str.value.clone(), DUMMY_SP, SyntaxContext::default())
                        }
                    };
                    let name = exported.as_ref().map_or_else(
                        || exported_ident.sym.as_str().to_string(),
                        |name| match name {
                            ModuleExportName::Ident(ident) => ident.sym.as_str().to_string(),
                            ModuleExportName::Str(str) => str.value.to_string(),
                        },
                    );

                    if export_named.src.is_none() {
                        exp_bindings.push(ExpBinding {
                            binding_ident: exp_binding_ident.clone(),
                            expr: Expr::from(exported_ident),
                        });
                        Some(ExpMember::new(exp_binding_ident, name))
                    } else {
                        Some(ExpMember::new(exported_ident, name))
                    }
                }
                ExportSpecifier::Namespace(ExportNamespaceSpecifier { name, .. }) => {
                    let exp_binding_ident = exp_binding_ident();
                    let exported_ident = match name {
                        ModuleExportName::Ident(ident) => ident.clone(),
                        ModuleExportName::Str(str) => {
                            Ident::new(str.value.clone(), DUMMY_SP, SyntaxContext::default())
                        }
                    };
                    let name: String = exported_ident.sym.as_str().to_string();

                    Some(ExpMember {
                        ident: exp_binding_ident,
                        name,
                        is_ns: true,
                    })
                }
                _ => None,
            })
            .collect::<Vec<ExpMember>>();

        if members.is_empty() {
            None
        } else {
            Some((
                if export_named.src.is_none() {
                    Exp::Default(DefaultExp::new(members))
                } else {
                    Exp::ReExport(ReExportExp::Named(
                        export_named.src.as_ref().unwrap().clone().value.to_string(),
                        members,
                    ))
                },
                exp_bindings,
            ))
        }
    }

    pub fn export_all_as_exp(export_all: &ExportAll) -> Exp {
        Exp::ReExport(ReExportExp::All(
            export_all.src.as_ref().clone().value.to_string(),
        ))
    }

    pub mod presets {
        use swc_core::{
            common::DUMMY_SP,
            ecma::{
                ast::*,
                utils::{member_expr, quote_ident, ExprFactory},
            },
        };

        use super::{arrow_with_paren_expr, obj_lit_expr, str_lit_expr, ExpMember};

        /// Returns the global module context declaration statement (register).
        ///
        /// ```js
        /// var ctx_ident = global.__modules.register(id);
        /// ```
        pub fn global_module_register_stmt(id: &String, ctx_ident: &Ident) -> Stmt {
            member_expr!(Default::default(), DUMMY_SP, global.__modules.register)
                .as_call(DUMMY_SP, vec![str_lit_expr(id).as_arg()])
                .into_var_decl(VarDeclKind::Var, ctx_ident.clone().into())
                .into()
        }

        /// Returns the global module context declaration statement (getContext).
        ///
        /// ```js
        /// var ctx_ident = global.__modules.getContext(id);
        /// ```
        pub fn global_module_get_ctx_stmt(id: &String, ctx_ident: &Ident) -> Stmt {
            member_expr!(Default::default(), DUMMY_SP, global.__modules.getContext)
                .as_call(DUMMY_SP, vec![str_lit_expr(id).as_arg()])
                .into_var_decl(VarDeclKind::Var, ctx_ident.clone().into())
                .into()
        }

        /// Returns the global module context reset call expression.
        ///
        /// ```js
        /// ctx_ident.reset();
        /// ```
        pub fn ctx_reset_call(ctx_ident: &Ident) -> Expr {
            ctx_ident
                .clone()
                .make_member(quote_ident!("reset"))
                .as_call(DUMMY_SP, vec![])
        }

        /// Returns a global module's require call expression.
        ///
        /// ```js
        /// // Code
        /// ctx_ident.require(src);
        /// ```
        pub fn require_call(ctx_ident: &Ident, src: Lit) -> Expr {
            ctx_ident
                .clone()
                .make_member(quote_ident!("require"))
                .as_call(DUMMY_SP, vec![src.as_arg()])
        }

        /// Returns a global module's exports call expression.
        ///
        /// ```js
        /// // Code
        /// ctx_ident.exports(function () {
        ///   return expr;
        /// });
        /// ```
        pub fn exports_call(ctx_ident: &Ident, exp_props: Vec<PropOrSpread>) -> Expr {
            ctx_ident
                .clone()
                .make_member(quote_ident!("exports"))
                .as_call(
                    DUMMY_SP,
                    vec![obj_lit_expr(exp_props).into_lazy_fn(vec![]).as_arg()],
                )
        }

        /// Returns the global module's require call and dependency declaration statement.
        ///
        /// ```js
        /// // Code
        /// // Pat: { foo, bar, default: baz }
        /// var { foo, bar, default: baz } = __ctx.require('./foo');
        /// ```
        pub fn decl_require_deps_stmt(ctx_ident: &Ident, src: Lit, pat: Pat) -> Stmt {
            require_call(ctx_ident, src)
                .into_var_decl(VarDeclKind::Var, pat)
                .into()
        }

        /// TODO
        ///
        /// ```js
        /// // Code
        /// global.__modules.define(function (context) {
        ///   // <stmts>
        /// }, id, {});
        /// ```
        pub fn define_call(
            id: &String,
            ctx_ident: &Ident,
            deps_ident: &Ident,
            stmts: Vec<Stmt>,
        ) -> Expr {
            member_expr!(Default::default(), DUMMY_SP, global.__modules.define).as_call(
                DUMMY_SP,
                vec![
                    Expr::Fn(FnExpr {
                        ident: None,
                        function: Box::new(Function {
                            params: vec![ctx_ident.clone().into()],
                            body: Some(BlockStmt {
                                stmts,
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                    .into(),
                    str_lit_expr(id).as_arg(),
                    deps_ident.clone().as_arg(),
                ],
            )
        }

        pub fn to_named_exps(exp_specs: Vec<ExportSpecifier>) -> ModuleItem {
            ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
                specifiers: exp_specs,
                type_only: false,
                src: None,
                with: None,
                span: DUMMY_SP,
            }))
        }

        pub fn to_dep_getter_expr(dep_member_props: &Vec<ObjectPatProp>) -> Expr {
            let dep_obj_props = dep_member_props
                .iter()
                .filter_map(|prop| match prop {
                    ObjectPatProp::KeyValue(KeyValuePatProp { key, value })
                        if matches!(&**value, Pat::Ident(_)) =>
                    {
                        match &**value {
                            Pat::Ident(ident) => Some(
                                Prop::KeyValue(KeyValueProp {
                                    key: key.clone(),
                                    value: Box::new(Expr::Ident(ident.clone().into())),
                                })
                                .into(),
                            ),
                            _ => None,
                        }
                    }
                    ObjectPatProp::Assign(AssignPatProp {
                        key, value: None, ..
                    }) => Some(Prop::Shorthand(key.clone().into()).into()),
                    _ => None,
                })
                .collect::<Vec<PropOrSpread>>();

            arrow_with_paren_expr(
                ObjectLit {
                    props: dep_obj_props,
                    ..Default::default()
                }
                .into(),
            )
        }

        pub fn to_deps_decl(dep_ident: &Ident, dep_getters: Vec<(String, Expr)>) -> Decl {
            Decl::Var(Box::new(VarDecl {
                decls: vec![VarDeclarator {
                    name: Pat::Ident(dep_ident.clone().into()),
                    definite: false,
                    init: Some(Box::new(Expr::Object(ObjectLit {
                        props: dep_getters
                            .iter()
                            .map(|(src, expr)| {
                                PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                    key: PropName::Str(src.clone().into()),
                                    value: Box::new(expr.clone()),
                                })))
                            })
                            .collect(),
                        ..Default::default()
                    }))),
                    span: DUMMY_SP,
                }],
                kind: VarDeclKind::Const,
                ..Default::default()
            }))
        }
    }
}

pub mod collections {
    use swc_core::common::collections::AHashMap;

    /// Ordered HashMap
    #[derive(Debug)]
    pub struct OHashMap<K, V> {
        map: AHashMap<K, V>,
        keys: Vec<K>,
    }

    impl<K: std::cmp::Eq + std::hash::Hash + Clone, V> OHashMap<K, V> {
        pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
            self.map.get_mut(key)
        }

        pub fn insert(&mut self, key: &K, value: V) {
            if !self.map.contains_key(&key) {
                self.keys.push(key.clone());
            }
            self.map.insert(key.clone(), value);
        }

        pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
            self.keys
                .iter()
                .filter_map(move |key| self.map.get(key).map(|value| (key, value)))
        }
    }

    impl<K: std::cmp::Eq + std::hash::Hash + Clone, V> Default for OHashMap<K, V> {
        fn default() -> Self {
            Self {
                map: AHashMap::default(),
                keys: Vec::new(),
            }
        }
    }
}
