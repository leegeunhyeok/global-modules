use std::path::PathBuf;

use swc_core::common::collections::AHashMap;
use swc_ecma_parser::{Syntax, TsSyntax};
use swc_ecma_transforms_testing::test_fixture;
use swc_global_modules::global_modules;

const MODULE_ID: f64 = 1000.0;

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

#[testing::fixture("tests/fixture/ids/**/input.js")]
fn ids_fixture(input: PathBuf) {
    let filename = input.to_string_lossy();
    let output = input.with_file_name("output.js");
    let phase = 1.0; // ModulePhase::Runtime

    let mut ids = AHashMap::default();
    ids.insert(String::from("react"), 1000 as f64);
    ids.insert(String::from("./foo"), 1001 as f64);
    ids.insert(String::from("./bar"), 1002 as f64);
    ids.insert(String::from("./baz"), 1003 as f64);
    ids.insert(String::from("./Component"), 1004 as f64);
    ids.insert(String::from("./cjs-1"), 1005 as f64);
    ids.insert(String::from("./cjs-2"), 1006 as f64);
    ids.insert(String::from("./cjs-3"), 1007 as f64);
    ids.insert(String::from("./esm"), 1008 as f64);
    ids.insert(String::from("./re-exp"), 1009 as f64);
    ids.insert(String::from("./re-exp-1"), 1010 as f64);
    ids.insert(String::from("./re-exp-2"), 1011 as f64);
    ids.insert(String::from("./re-exp-3"), 1012 as f64);
    ids.insert(String::from("./re-exp-4"), 1013 as f64);
    ids.insert(String::from("./re-exp-5"), 1014 as f64);

    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: filename.ends_with(".tsx"),
            ..Default::default()
        }),
        &|_| global_modules(MODULE_ID, phase, ids.clone().into()),
        &input,
        &output,
        Default::default(),
    );
}
