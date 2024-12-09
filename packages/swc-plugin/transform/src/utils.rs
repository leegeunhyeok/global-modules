pub mod ast {
    use swc_core::{
        atoms::Atom,
        common::DUMMY_SP,
        ecma::{
            ast::*,
            utils::{member_expr, quote_ident, ExprFactory},
        },
    };

    use crate::models::ExportMember;

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

    pub fn spread_prop(expr: Expr) -> PropOrSpread {
        PropOrSpread::Spread(SpreadElement {
            expr: expr.into(),
            ..Default::default()
        })
    }

    pub fn assign_expr(ident: &Ident, expr: Expr) -> AssignExpr {
        AssignExpr {
            left: AssignTarget::Simple(SimpleAssignTarget::Ident(ident.clone().into())),
            right: expr.into(),
            op: AssignOp::Assign,
            ..Default::default()
        }
    }

    pub fn obj_lit_expr(props: Vec<PropOrSpread>) -> Expr {
        Expr::Object(ObjectLit {
            props,
            ..Default::default()
        })
    }

    pub fn num_lit_expr(num: f64) -> Expr {
        Expr::Lit(Lit::Num(Number {
            span: DUMMY_SP,
            value: num,
            raw: None,
        }))
    }

    pub fn var_declarator(ident: &Ident) -> VarDeclarator {
        VarDeclarator {
            name: Pat::Ident(ident.clone().into()),
            definite: false,
            init: None,
            span: DUMMY_SP,
        }
    }

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

    /// Wraps the given expression with a `__ctx.ns` function call expression.
    ///
    /// ```js
    /// // Code
    /// __ctx.ns(<expr>);
    /// ```
    pub fn to_ns_export(ident: &Ident, expr: Expr) -> Expr {
        ident
            .clone()
            .make_member(quote_ident!("ns"))
            .as_call(DUMMY_SP, vec![expr.into()])
    }

    pub fn expr_from_export_default_decl(
        export_default_decl: &ExportDefaultDecl,
        ident: Ident,
    ) -> Expr {
        assign_expr(
            &ident,
            get_expr_from_default_decl(&export_default_decl.decl),
        )
        .into()
    }

    pub fn default_expr_from_default_export_decl(
        export_default_decl: &ExportDefaultDecl,
        ident: Ident,
    ) -> ModuleItem {
        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
            expr: assign_expr(
                &ident,
                get_expr_from_default_decl(&export_default_decl.decl),
            )
            .into(),
            span: DUMMY_SP,
        }))
        .into()
    }

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

    pub fn get_expr_from_default_decl(default_decl: &DefaultDecl) -> Expr {
        match default_decl {
            DefaultDecl::Class(class_expr) => Expr::Class(class_expr.clone()),
            DefaultDecl::Fn(fn_expr) => Expr::Fn(fn_expr.clone()),
            _ => panic!("not implemented"),
        }
    }

    pub fn to_export_members(specifiers: &Vec<ExportSpecifier>) -> Vec<ExportMember> {
        let mut members = Vec::with_capacity(specifiers.len());

        specifiers.iter().for_each(|spec| match spec {
            ExportSpecifier::Named(
                specifier @ ExportNamedSpecifier {
                    is_type_only: false,
                    ..
                },
            ) => members.push(specifier.into()),
            _ => {}
        });

        members
    }

    pub fn global_module_register_stmt(id: f64, ctx_ident: &Ident) -> ModuleItem {
        member_expr!(Default::default(), DUMMY_SP, global.__modules.register)
            .as_call(DUMMY_SP, vec![num_lit_expr(id).as_arg()])
            .into_var_decl(VarDeclKind::Var, ctx_ident.clone().into())
            .into()
    }

    pub fn global_module_get_ctx_stmt(id: f64, ctx_ident: &Ident) -> ModuleItem {
        member_expr!(Default::default(), DUMMY_SP, global.__modules.getContext)
            .as_call(DUMMY_SP, vec![num_lit_expr(id).as_arg()])
            .into_var_decl(VarDeclKind::Var, ctx_ident.clone().into())
            .into()
    }

    /// Returns global module's require call expression.
    ///
    /// ```js
    /// // Code
    /// ident.require(src);
    /// ```
    pub fn require_call(ident: &Ident, src: &Atom) -> Expr {
        ident
            .clone()
            .make_member(quote_ident!("require"))
            .as_call(DUMMY_SP, vec![src.clone().as_arg()])
    }

    /// Returns global module's exports call expression.
    ///
    /// ```js
    /// // Code
    /// ident.exports(function () {
    ///   return <obj>;
    /// });
    /// ```
    pub fn exports_call(ident: &Ident, obj: Expr) -> Expr {
        ident
            .clone()
            .make_member(quote_ident!("exports"))
            .as_call(DUMMY_SP, vec![obj.into_lazy_fn(vec![]).as_arg()])
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
