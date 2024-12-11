pub mod ast {
    use swc_core::{
        atoms::Atom,
        common::DUMMY_SP,
        ecma::{ast::*, utils::ExprFactory},
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

    /// Returns a number literal expression.
    ///
    /// ```js
    /// // Code
    /// 100
    /// ```
    pub fn num_lit_expr(num: f64) -> Expr {
        Expr::Lit(Lit::Num(Number {
            span: DUMMY_SP,
            value: num,
            raw: None,
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
    /// // ModulePhase::Register
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

        match &assign_expr.left {
            AssignTarget::Simple(SimpleAssignTarget::Member(member_expr)) => {
                if member_expr.obj.is_ident_ref_to("exports") {
                    // `exports.foo = ...;`
                    let new_assign_expr = assign_member(
                        ctx_ident
                            .make_member(IdentName {
                                sym: "exports".into(),
                                ..Default::default()
                            })
                            .make_member(IdentName {
                                sym: member_expr.prop.as_ident().unwrap().sym.clone(),
                                ..Default::default()
                            }),
                        *assign_expr.right.clone(),
                    );

                    if phase == ModulePhase::Register {
                        new_assign_expr.make_assign_to(AssignOp::Assign, assign_expr.left.clone())
                    } else {
                        new_assign_expr
                    }
                    .into()
                } else if is_cjs_module_member(member_expr) {
                    // `module.exports = ...;`
                    let new_assign_expr = assign_member(
                        ctx_ident
                            .make_member(IdentName {
                                sym: "module".into(),
                                ..Default::default()
                            })
                            .make_member(IdentName {
                                sym: "exports".into(),
                                ..Default::default()
                            }),
                        *assign_expr.right.clone(),
                    );

                    if phase == ModulePhase::Register {
                        new_assign_expr.make_assign_to(AssignOp::Assign, assign_expr.left.clone())
                    } else {
                        new_assign_expr
                    }
                    .into()
                } else if let Some(inner_member_expr) = member_expr.obj.as_member() {
                    if is_cjs_module_member(inner_member_expr) {
                        // `module.exports.foo = ...;`
                        let new_assign_expr = assign_member(
                            ctx_ident
                                .make_member(IdentName {
                                    sym: "exports".into(),
                                    ..Default::default()
                                })
                                .make_member(IdentName {
                                    sym: member_expr.prop.as_ident().unwrap().sym.clone(),
                                    ..Default::default()
                                }),
                            *assign_expr.right.clone(),
                        );

                        if phase == ModulePhase::Register {
                            new_assign_expr
                                .make_assign_to(AssignOp::Assign, assign_expr.left.clone())
                        } else {
                            new_assign_expr
                        }
                        .into()
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
    /// // ModulePhase::Register
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

        if phase == ModulePhase::Register {
            assign_member(member_expr.clone(), ctx_module_member.into())
        } else {
            Expr::from(ctx_module_member)
        }
        .into()
    }

    /// Returns an expression that binds the export default declaration
    /// to the provided identifier.
    ///
    /// ```js
    /// // Input
    /// export default function foo() {};
    /// export default class Bar {};
    ///
    /// // Code
    /// ident = function foo() {};
    /// ident = class Bar {};
    /// ```
    pub fn expr_from_export_default_decl(
        export_default_decl: &ExportDefaultDecl,
        ident: Ident,
    ) -> Expr {
        assign_expr(ident, get_expr_from_default_decl(&export_default_decl.decl)).into()
    }

    /// Returns an export default expression bound to the
    /// provided identifier from an export default statement.
    ///
    /// ```js
    /// // Input
    /// export default function foo() {};
    /// export default class Bar {};
    ///
    /// // Code
    /// export default ident = function foo() {};
    /// export default ident = class Bar {};
    /// ```
    pub fn default_expr_from_default_export_decl(
        export_default_decl: &ExportDefaultDecl,
        ident: Ident,
    ) -> ModuleItem {
        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
            expr: assign_expr(ident, get_expr_from_default_decl(&export_default_decl.decl)).into(),
            span: DUMMY_SP,
        }))
        .into()
    }

    /// Extracts and returns the expression with its ident from the declarations.
    ///
    /// ```js
    /// function foo {}
    /// class Bar {}
    /// const baz = expr;
    /// ```
    pub fn get_expr_from_decl(decl: &Decl) -> (Ident, Expr) {
        match decl {
            Decl::Class(ClassDecl {
                class,
                ident,
                declare: false,
            }) => (
                ident.clone(),
                ClassExpr {
                    class: class.clone(),
                    ident: Some(ident.clone()),
                }
                .into(),
            ),
            Decl::Fn(FnDecl {
                function,
                ident,
                declare: false,
            }) => (
                ident.clone(),
                FnExpr {
                    function: function.clone(),
                    ident: Some(ident.clone()),
                }
                .into(),
            ),
            Decl::Var(val_decl) => {
                if val_decl.decls.len() != 1 {
                    panic!("invalid named exports");
                }

                let var_decl = val_decl.decls.get(0).unwrap();

                match var_decl {
                    VarDeclarator {
                        name: Pat::Ident(BindingIdent { id, type_ann: None }),
                        init: Some(expr),
                        definite: false,
                        ..
                    } => (id.clone(), *expr.clone()),
                    _ => panic!("invalid"),
                }
            }
            _ => panic!("invalid"),
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
            _ => panic!("not implemented"),
        }
    }

    pub mod presets {
        use swc_core::{
            atoms::Atom,
            common::DUMMY_SP,
            ecma::{
                ast::*,
                utils::{member_expr, quote_ident, ExprFactory},
            },
        };

        use super::num_lit_expr;

        /// Returns the global module context declaration statement (register).
        ///
        /// ```js
        /// var ctx_ident = global.__modules.register(id);
        /// ```
        pub fn global_module_register_stmt(id: f64, ctx_ident: &Ident) -> ModuleItem {
            member_expr!(Default::default(), DUMMY_SP, global.__modules.register)
                .as_call(DUMMY_SP, vec![num_lit_expr(id).as_arg()])
                .into_var_decl(VarDeclKind::Var, ctx_ident.clone().into())
                .into()
        }

        /// Returns the global module context declaration statement (getContext).
        ///
        /// ```js
        /// var ctx_ident = global.__modules.getContext(id);
        /// ```
        pub fn global_module_get_ctx_stmt(id: f64, ctx_ident: &Ident) -> ModuleItem {
            member_expr!(Default::default(), DUMMY_SP, global.__modules.getContext)
                .as_call(DUMMY_SP, vec![num_lit_expr(id).as_arg()])
                .into_var_decl(VarDeclKind::Var, ctx_ident.clone().into())
                .into()
        }

        /// Returns a global module's require call expression.
        ///
        /// ```js
        /// // Code
        /// ctx_ident.require(src);
        /// ```
        pub fn require_call(ctx_ident: Ident, src: Atom) -> Expr {
            ctx_ident
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
        /// var { foo, bar, default: baz } = ctx_ident.require('./foo');
        /// ```
        pub fn decl_require_deps_stmt(ctx_ident: Ident, src: Atom, pat: Pat) -> Stmt {
            require_call(ctx_ident, src)
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
