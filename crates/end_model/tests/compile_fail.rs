#[test]
fn cross_catalog_ids_do_not_typecheck() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/cross_catalog_item_id.rs");
}
