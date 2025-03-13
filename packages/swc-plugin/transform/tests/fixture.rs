use std::path::PathBuf;

use swc_core::{
    common::{collections::AHashMap, Mark, SyntaxContext},
    ecma::{ast::Pass, transforms::base::resolver, visit::VisitMut},
};
use swc_ecma_parser::{Syntax, TsSyntax};
use swc_ecma_transforms_testing::test_fixture;
use swc_global_modules::global_modules;

const MODULE_ID: &str = "1000";

fn tr(phase: f64) -> impl VisitMut + Pass {
    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();

    (
        resolver(unresolved_mark, top_level_mark, false),
        global_modules(
            String::from(MODULE_ID),
            phase,
            SyntaxContext::empty().apply_mark(unresolved_mark),
        ),
    )
}

#[testing::fixture("tests/fixture/bundle/**/input.js")]
fn bundle_fixture(input: PathBuf) {
    let filename = input.to_string_lossy();
    let output = input.with_file_name("output.js");
    let phase = 0.0; // ModulePhase::Bundle

    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: filename.ends_with(".tsx"),
            ..Default::default()
        }),
        &|_| tr(phase),
        &input,
        &output,
        Default::default(),
    );
}

#[testing::fixture("tests/fixture/runtime/**/input.js")]
fn runtime_fixture(input: PathBuf) {
    let filename = input.to_string_lossy();
    let output = input.with_file_name("output.js");
    let phase = 1.0; // ModulePhase::Runtime

    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: filename.ends_with(".tsx"),
            ..Default::default()
        }),
        &|_| tr(phase),
        &input,
        &output,
        Default::default(),
    );
}
