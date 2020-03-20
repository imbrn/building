#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/parsing.rs");
    t.pass("tests/creating_builder.rs");
    t.pass("tests/calling_setters.rs");
    t.pass("tests/calling_build.rs");
    t.pass("tests/optional_fields.rs");
}
