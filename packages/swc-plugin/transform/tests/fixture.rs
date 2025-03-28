use std::path::PathBuf;

use swc_core::{
    common::{collections::AHashMap, Mark, SyntaxContext},
    ecma::{ast::Pass, transforms::base::resolver, visit::VisitMut},
};
use swc_ecma_parser::{Syntax, TsSyntax};
use swc_ecma_transforms_testing::test_fixture;
use swc_global_modules::global_modules;

const MODULE_ID: &str = "1000";

fn tr(runtime: bool, paths: Option<AHashMap<String, String>>) -> impl VisitMut + Pass {
    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();

    (
        resolver(unresolved_mark, top_level_mark, false),
        global_modules(
            String::from(MODULE_ID),
            runtime,
            paths,
            SyntaxContext::empty().apply_mark(unresolved_mark),
        ),
    )
}

#[testing::fixture("tests/fixture/bundle/**/input.js")]
fn bundle_fixture(input: PathBuf) {
    let filename = input.to_string_lossy();
    let output = input.with_file_name("output.js");
    let runtime = false;

    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: filename.ends_with(".tsx"),
            ..Default::default()
        }),
        &|_| tr(runtime, None),
        &input,
        &output,
        Default::default(),
    );
}

#[testing::fixture("tests/fixture/runtime/**/input.js")]
fn runtime_fixture(input: PathBuf) {
    let filename = input.to_string_lossy();
    let output = input.with_file_name("output.js");
    let runtime = true;

    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: filename.ends_with(".tsx"),
            ..Default::default()
        }),
        &|_| tr(runtime, None),
        &input,
        &output,
        Default::default(),
    );
}

#[testing::fixture("tests/fixture/paths/**/input.js")]
fn paths_fixture(input: PathBuf) {
    let filename = input.to_string_lossy();
    let output = input.with_file_name("output.js");
    let runtime = true;

    let mut paths = AHashMap::default();
    paths.insert(String::from("react"), String::from("1000"));
    paths.insert(String::from("./foo"), String::from("1001"));
    paths.insert(String::from("./bar"), String::from("1002"));
    paths.insert(String::from("./baz"), String::from("1003"));
    paths.insert(String::from("./Component"), String::from("1004"));
    paths.insert(String::from("./cjs-1"), String::from("1005"));
    paths.insert(String::from("./cjs-2"), String::from("1006"));
    paths.insert(String::from("./cjs-3"), String::from("1007"));
    paths.insert(String::from("./esm"), String::from("1008"));
    paths.insert(String::from("./re-exp"), String::from("1009"));
    paths.insert(String::from("./re-exp-1"), String::from("1010"));
    paths.insert(String::from("./re-exp-2"), String::from("1011"));
    paths.insert(String::from("./re-exp-3"), String::from("1012"));
    paths.insert(String::from("./re-exp-4"), String::from("1013"));
    paths.insert(String::from("./re-exp-5"), String::from("1014"));

    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: filename.ends_with(".tsx"),
            ..Default::default()
        }),
        &|_| tr(runtime, Some(paths.clone())),
        &input,
        &output,
        Default::default(),
    );
}
