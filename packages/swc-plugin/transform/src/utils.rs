use core::panic;

use swc_core::{
    atoms::Atom,
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::{ExprFactory, FunctionFactory},
    },
};

use crate::module_collector::ModuleMember;

/// ```js
/// var { foo, bar, default: baz } = __require('./foo', false);
/// ```
pub fn get_require_call_expr(
    require_ident: &Ident,
    src: &Atom,
    modules: &Vec<ModuleMember>,
) -> ModuleItem {
    let deps_props = modules
        .iter()
        .map(|imp_member| match &imp_member.alias {
            Some(alias_ident) => ObjectPatProp::KeyValue(KeyValuePatProp {
                key: PropName::Ident(imp_member.ident.clone().into()),
                value: Box::new(Pat::Ident(alias_ident.clone().into())),
            }),
            None => ObjectPatProp::Assign(AssignPatProp {
                key: imp_member.ident.clone().into(),
                value: None,
                span: DUMMY_SP,
            }),
        })
        .collect::<Vec<ObjectPatProp>>();

    get_require_expr(require_ident, src, false)
        .into_var_decl(
            VarDeclKind::Var,
            ObjectPat {
                props: deps_props,
                optional: false,
                type_ann: None,
                span: DUMMY_SP,
            }
            .into(),
        )
        .into()
}

/// ```js
/// var foo = __require('./foo', true);
/// ```
pub fn get_ns_require_call_expr(
    require_ident: &Ident,
    src: &Atom,
    module: &ModuleMember,
) -> ModuleItem {
    get_require_expr(&require_ident, &src, true)
        .into_var_decl(VarDeclKind::Var, module.ident.clone().into())
        .into()
}

/// ```js
/// __require('./foo', is_star);
/// ```
pub fn get_require_expr(require_ident: &Ident, src: &Atom, is_star: bool) -> Expr {
    let mut require_args = vec![src.clone().as_arg()];

    if is_star {
        require_args.push(
            Lit::Bool(Bool {
                span: DUMMY_SP,
                value: true,
            })
            .as_arg(),
        );
    }

    require_ident.clone().as_call(DUMMY_SP, require_args)
}

pub fn get_expr_from_decl(decl: &Decl) -> Expr {
    match decl {
        Decl::Class(ClassDecl {
            class,
            ident,
            declare: false,
        }) => ClassExpr {
            class: class.clone(),
            ident: Some(ident.clone()),
        }
        .into(),
        Decl::Fn(FnDecl {
            function,
            ident,
            declare: false,
        }) => FnExpr {
            function: function.clone(),
            ident: Some(ident.clone()),
        }
        .into(),
        Decl::Var(val_decl) => {
            if val_decl.decls.len() != 1 {
                panic!("invalid named exports");
            }

            if let Some(init) = &val_decl.decls.get(0).unwrap().init {
                *init.clone()
            } else {
                panic!("invalid");
            }
        }
        _ => panic!("not implemented"),
    }
}

pub fn get_expr_from_default_decl(default_decl: &DefaultDecl) -> Expr {
    match default_decl {
        DefaultDecl::Class(class_expr) => Expr::Class(class_expr.clone()),
        DefaultDecl::Fn(fn_expr) => Expr::Fn(fn_expr.clone()),
        _ => panic!("not implemented"),
    }
}

/// Wrap expression with function.
///
/// ```js
/// function () {
///   return /* expr */;
/// }
/// ```
pub fn wrap_with_fn(expr: &Expr) -> Expr {
    Function {
        body: Some(BlockStmt {
            stmts: vec![Stmt::Return(ReturnStmt {
                arg: Some(Box::new(expr.clone())),
                ..Default::default()
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
    .into()
}
