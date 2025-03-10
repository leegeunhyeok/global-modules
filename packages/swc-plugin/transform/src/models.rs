use presets::require_call;
use swc_core::{
    common::DUMMY_SP,
    ecma::{ast::*, utils::ExprFactory},
};

use crate::utils::ast::*;

// Dependency of module
#[derive(Debug)]
pub enum Dep {
    Default(DefaultDep),
    Lazy(LazyDep),
}

impl Dep {
    pub fn default(src: String, members: Vec<DepMember>) -> Self {
        Dep::Default(DefaultDep { src, members })
    }

    pub fn lazy(src: String, expr: Expr) -> Self {
        Dep::Lazy(LazyDep { src, expr })
    }
}

#[derive(Debug)]
pub struct DefaultDep {
    pub src: String,
    pub members: Vec<DepMember>,
}

#[derive(Debug)]
pub struct DepMember {
    pub ident: Ident,
    pub name: Option<String>,
}

impl DepMember {
    pub fn new(ident: Ident, name: Option<String>) -> Self {
        DepMember { ident, name }
    }

    pub fn into_obj_pat_prop(self) -> ObjectPatProp {
        match self.name {
            Some(name) => obj_kv_prop(name.into(), self.ident.into()),
            None => obj_assign_prop(self.ident.into()),
        }
    }
}

#[derive(Debug)]
pub struct LazyDep {
    pub src: String,
    pub expr: Expr,
}

#[derive(Debug)]
pub enum Exp {
    Default(DefaultExp),
    ReExportAll(ReExportAllExp),
    ReExportNamed(ReExportNamedExp),
}

#[derive(Debug)]
pub struct DefaultExp {
    pub members: Vec<ExpMember>,
}

impl DefaultExp {
    pub fn new(members: Vec<ExpMember>) -> Self {
        Self { members }
    }

    pub fn into_exp_ast(self) -> (Vec<VarDeclarator>, Vec<PropOrSpread>, Vec<ExportSpecifier>) {
        let len = self.members.len();
        let mut declarators = Vec::with_capacity(len);
        let mut props = Vec::with_capacity(len);
        let mut specs = Vec::with_capacity(len);

        self.members.into_iter().for_each(|member| {
            declarators.push(var_declarator(member.ident.clone().into(), None));
            props.push(kv_prop(
                member.name.clone().into(),
                member.ident.clone().into(),
            ));
            specs.push(ExportSpecifier::Named(ExportNamedSpecifier {
                orig: ModuleExportName::Ident(member.ident),
                exported: Some(ModuleExportName::Ident(Ident::from(member.name))),
                is_type_only: false,
                span: DUMMY_SP,
            }));
        });

        (declarators, props, specs)
    }
}

#[derive(Debug)]
pub struct ReExportAllExp {
    pub src: String,
    pub alias: Option<Ident>,
}

impl ReExportAllExp {
    pub fn default(src: String) -> Self {
        Self { src, alias: None }
    }

    pub fn alias(src: String, ident: Ident) -> Self {
        Self {
            src,
            alias: Some(ident),
        }
    }

    pub fn get_src(&self) -> String {
        self.src.clone()
    }

    pub fn to_import_stmt(&self, mod_ident: Ident) -> ModuleItem {
        import_star(mod_ident, self.get_src().into())
    }

    pub fn to_require_stmt(&self, ctx_ident: &Ident, mod_ident: Ident) -> Stmt {
        require_call(ctx_ident, self.get_src().clone().into())
            .into_var_decl(VarDeclKind::Const, mod_ident.into())
            .into()
    }
    pub fn to_exp_props(&self, ctx_ident: &Ident, mod_ident: Ident) -> PropOrSpread {
        match &self.alias {
            Some(ident) => kv_prop(ident.sym.clone(), mod_ident.into()),
            None => spread_prop(to_ns_export(ctx_ident.clone().into(), mod_ident.into())),
        }
    }
}

#[derive(Debug)]
pub struct ReExportNamedExp {
    pub src: String,
    pub members: Vec<ExpMember>,
}

impl ReExportNamedExp {
    fn get_src(&self) -> String {
        self.src.clone()
    }

    pub fn to_import_stmt(&self, mod_ident: Ident) -> ModuleItem {
        ImportDecl {
            src: Box::new(self.get_src().into()),
            specifiers: vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
                local: mod_ident,
                span: DUMMY_SP,
            })],
            phase: ImportPhase::Evaluation,
            type_only: false,
            with: None,
            span: DUMMY_SP,
        }
        .into()
    }

    pub fn to_require_stmt(&self, ctx_ident: &Ident, mod_ident: Ident) -> Stmt {
        require_call(ctx_ident, self.get_src().clone().into())
            .into_var_decl(VarDeclKind::Const, mod_ident.into())
            .into()
    }

    pub fn to_exp_props(&self, mod_ident: Ident) -> Vec<PropOrSpread> {
        self.members
            .iter()
            .map(|member| {
                PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(IdentName {
                        sym: member.name.clone().into(),
                        span: DUMMY_SP,
                    }),
                    value: Box::new(
                        mod_ident
                            .clone()
                            .make_member(IdentName {
                                sym: member.ident.sym.clone(),
                                ..Default::default()
                            })
                            .into(),
                    ),
                })))
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct ExpMember {
    pub ident: Ident,
    pub name: String,
}

impl ExpMember {
    pub fn new(ident: Ident, name: String) -> Self {
        Self { ident, name }
    }
}

#[derive(Debug)]
pub struct ExpBinding {
    pub binding_ident: Ident,
    pub expr: Expr,
}

impl ExpBinding {
    pub fn to_assign_expr(self) -> Expr {
        assign_expr(self.binding_ident, self.expr).into()
    }
}
