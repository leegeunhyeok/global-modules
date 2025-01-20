pub mod ast {
    use swc_core::{
        atoms::Atom,
        common::{collections::AHashMap, Spanned, DUMMY_SP},
        ecma::{ast::*, utils::ExprFactory},
        plugin::errors::HANDLER,
    };

    use crate::phase::ModulePhase;

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
    /// module.exports; // true;
    /// ```
    pub fn is_cjs_module_member(member_expr: &MemberExpr) -> bool {
        member_expr.obj.is_ident_ref_to("module") && member_expr.prop.is_ident_with("exports")
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
        phase: ModulePhase,
    ) -> Option<Expr> {
        if assign_expr.op != AssignOp::Assign {
            return None;
        }

        let get_new_assign_expr = |expr: Expr, named_sym: Option<&str>| {
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

            let new_assign_expr = assign_member(
                if let Some(named_sym) = named_sym {
                    // `named_sym`: foo
                    // => `ctx_ident.module.foo`
                    ctx_module.make_member(IdentName {
                        sym: named_sym.into(),
                        ..Default::default()
                    })
                } else {
                    ctx_module
                },
                expr,
            );

            if phase == ModulePhase::Bundle {
                new_assign_expr.make_assign_to(AssignOp::Assign, assign_expr.left.clone())
            } else {
                new_assign_expr
            }
            .into()
        };

        match &assign_expr.left {
            AssignTarget::Simple(SimpleAssignTarget::Member(member_expr)) => {
                if member_expr.obj.is_ident_ref_to("exports") && member_expr.prop.is_ident() {
                    // `exports.foo = ...;`
                    // `exports['foo'] = ...;`
                    get_new_assign_expr(
                        *assign_expr.right.clone(),
                        get_sym_from_member_prop(&member_expr.prop).into(),
                    )
                } else if is_cjs_module_member(member_expr) {
                    // `module.exports = ...;`
                    get_new_assign_expr(*assign_expr.right.clone(), None)
                } else if let Some(leading_member) = member_expr.obj.as_member() {
                    if is_cjs_module_member(leading_member) {
                        // `module.exports.foo = ...;`
                        // `module.exports['foo'] = ...;`
                        get_new_assign_expr(
                            *assign_expr.right.clone(),
                            get_sym_from_member_prop(&member_expr.prop).into(),
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
        phase: ModulePhase,
    ) -> Option<Expr> {
        if is_cjs_module_member(member_expr) == false {
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

        if phase == ModulePhase::Bundle {
            assign_member(member_expr.clone(), ctx_module_member.into())
        } else {
            Expr::from(ctx_module_member)
        }
        .into()
    }

    /// Extracts and returns the its ident from the declarations.
    ///
    /// ```js
    /// function foo {}
    /// class Bar {}
    /// const baz = expr;
    /// ```
    pub fn get_ident_from_decl(decl: &Decl) -> Ident {
        match decl {
            Decl::Class(ClassDecl {
                ident,
                declare: false,
                ..
            }) => ident.clone(),
            Decl::Fn(FnDecl {
                ident,
                declare: false,
                ..
            }) => ident.clone(),
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
                    } => id.clone(),
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
            _ => {
                HANDLER.with(|handler| {
                    handler
                        .struct_span_err(decl.span(), "unsupported declaration")
                        .emit();
                });
                panic!(); // FIXME
            }
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

    pub fn get_sym_from_member_prop(prop: &MemberProp) -> &str {
        match prop {
            MemberProp::Ident(ident) => ident.sym.as_str(),
            MemberProp::Computed(ComputedPropName { expr, .. }) => match &**expr {
                Expr::Lit(Lit::Str(str_lit)) => str_lit.value.as_str(),
                _ => HANDLER.with(|handler| {
                    handler
                        .struct_span_err(prop.span(), "invalid expression for computed property")
                        .emit();

                    ""
                }),
            },
            _ => HANDLER.with(|handler| {
                handler
                    .struct_span_err(prop.span(), "unsupported property type")
                    .emit();

                ""
            }),
        }
    }

    pub mod presets {
        use swc_core::{
            common::DUMMY_SP,
            ecma::{
                ast::*,
                utils::{member_expr, quote_ident, ExprFactory},
            },
        };

        use super::str_lit_expr;

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
        /// global.__modules.require(src);
        /// ```
        pub fn require_call(src: Lit) -> Expr {
            member_expr!(Default::default(), DUMMY_SP, global.__modules)
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
        pub fn exports_call(ctx_ident: Ident, expr: Expr) -> Expr {
            ctx_ident
                .make_member(quote_ident!("exports"))
                .as_call(DUMMY_SP, vec![expr.into_lazy_fn(vec![]).as_arg()])
        }

        /// Returns the global module's require call and dependency declaration statement.
        ///
        /// ```js
        /// // Code
        /// // Pat: { foo, bar, default: baz }
        /// var { foo, bar, default: baz } = global.__modules.require('./foo');
        /// ```
        pub fn decl_require_deps_stmt(src: Lit, pat: Pat) -> Stmt {
            require_call(src)
                .into_var_decl(VarDeclKind::Var, pat)
                .into()
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
