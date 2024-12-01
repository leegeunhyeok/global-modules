pub mod ast {
    use swc_core::{atoms::Atom, common::DUMMY_SP, ecma::ast::*};

    pub fn kv_prop(key: &Atom, value: Expr) -> PropOrSpread {
        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: PropName::Str(Str {
                value: key.clone(),
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

    pub fn num_lit_expr(num: u32) -> Expr {
        Expr::Lit(Lit::Num(Number {
            span: DUMMY_SP,
            value: num as f64,
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

    pub fn import_star(ident: &Ident, src: &Atom) -> ModuleItem {
        ModuleDecl::Import(ImportDecl {
            phase: ImportPhase::Evaluation,
            specifiers: vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
                local: ident.clone(),
                span: DUMMY_SP,
            })],
            src: Box::new(src.clone().into()),
            type_only: false,
            with: None,
            span: DUMMY_SP,
        })
        .into()
    }
}

pub mod parse {
    use swc_core::ecma::ast::*;

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
}
