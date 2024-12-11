use std::path::PathBuf;

use swc_ecma_parser::{Syntax, TsSyntax};
use swc_ecma_transforms_testing::test_fixture;
use swc_global_modules::global_modules;

const MODULE_ID: f64 = 1000.0;

#[testing::fixture("tests/fixture/register/**/input.js")]
fn register_fixture(input: PathBuf) {
    let filename = input.to_string_lossy();
    let output = input.with_file_name("output.js");
    let phase = 0.0; // ModulePhase::Register

    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: filename.ends_with(".tsx"),
            ..Default::default()
        }),
        &|_| global_modules(MODULE_ID, phase, None),
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
        &|_| global_modules(MODULE_ID, phase, None),
        &input,
        &output,
        Default::default(),
    );
}
