use end_io::{Error, default_aic_toml, load_aic, load_catalog};
use end_model::{AicInputs, Catalog};
use generativity::make_guard;
use std::fs;
use tempfile::TempDir;

fn load_aic_from_str<'id>(src: &str, catalog: &Catalog<'id>) -> Result<AicInputs<'id>, Error> {
    let dir = TempDir::new().expect("create temp dir");
    let aic_path = dir.path().join("aic.toml");
    fs::write(&aic_path, src).expect("write aic.toml");
    load_aic(&aic_path, catalog)
}

fn first_two_item_keys<'a, 'id>(catalog: &'a Catalog<'id>) -> (&'a str, &'a str) {
    assert!(
        catalog.items().len() >= 2,
        "builtin catalog must contain at least two items"
    );
    (&catalog.items()[0].key, &catalog.items()[1].key)
}

fn assert_schema_location(
    err: &Error,
    expected_path_suffix: &str,
    expected_field: &str,
    expected_index: Option<usize>,
) {
    match err {
        Error::Schema {
            path,
            field,
            index,
            message,
        } => {
            assert!(
                path.ends_with(expected_path_suffix),
                "unexpected path: {path:?}"
            );
            assert_eq!(field, expected_field, "unexpected field");
            assert_eq!(*index, expected_index, "unexpected index");
            assert!(!message.is_empty(), "schema message should not be empty");
        }
        _ => panic!("unexpected error: {err:?}"),
    }
}

#[test]
fn load_aic_rejects_unknown_supply_item() {
    make_guard!(guard);
    let catalog = load_catalog(None, guard).expect("load builtin catalog");
    let err = load_aic_from_str(
        r#"
external_power_consumption_w = 0

[supply_per_min]
"Unknown Item" = 1
"#,
        &catalog,
    )
    .expect_err("unknown item should fail");
    assert!(
        matches!(err, Error::UnknownItem { ref key, .. } if key == "Unknown Item"),
        "unexpected error: {err:?}"
    );
}

#[test]
fn load_aic_rejects_duplicate_outpost_keys() {
    make_guard!(guard);
    let catalog = load_catalog(None, guard).expect("load builtin catalog");
    let (_, price_item) = first_two_item_keys(&catalog);
    let src = format!(
        r#"
external_power_consumption_w = 0

[[outposts]]
key = "Dup"
money_cap_per_hour = 60
prices = {{ "{price_item}" = 1 }}

[[outposts]]
key = "Dup"
money_cap_per_hour = 120
prices = {{ "{price_item}" = 2 }}
"#
    );

    let err = load_aic_from_str(&src, &catalog).expect_err("duplicate outpost key should fail");
    assert!(
        matches!(
            err,
            Error::DuplicateKey {
                ref kind,
                ref key,
                ..
            } if kind == "outpost" && key == "Dup"
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn load_aic_rejects_zero_supply_value() {
    make_guard!(guard);
    let catalog = load_catalog(None, guard).expect("load builtin catalog");
    let (supply_item, _) = first_two_item_keys(&catalog);
    let src = format!(
        r#"
external_power_consumption_w = 0

[supply_per_min]
"{supply_item}" = 0
"#
    );

    let err = load_aic_from_str(&src, &catalog).expect_err("zero supply should fail");
    assert!(
        matches!(
            err,
            Error::Schema {
                ref field,
                ref message,
                ..
            } if field == "supply_per_min.value" && message.contains("must be >= 1")
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn load_aic_rejects_negative_external_power() {
    make_guard!(guard);
    let catalog = load_catalog(None, guard).expect("load builtin catalog");
    let src = r#"
external_power_consumption_w = -1
"#;

    let err = load_aic_from_str(src, &catalog).expect_err("negative external power should fail");
    assert!(
        matches!(
            err,
            Error::Schema {
                ref field,
                ref message,
                ..
            } if field == "external_power_consumption_w" && message.contains("must be >= 0")
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn load_aic_rejects_negative_outpost_price() {
    make_guard!(guard);
    let catalog = load_catalog(None, guard).expect("load builtin catalog");
    let (_, price_item) = first_two_item_keys(&catalog);
    let src = format!(
        r#"
external_power_consumption_w = 0

[[outposts]]
key = "Camp"
money_cap_per_hour = 60
prices = {{ "{price_item}" = -1 }}
"#
    );

    let err = load_aic_from_str(&src, &catalog).expect_err("negative price should fail");
    assert!(
        matches!(
            err,
            Error::Schema {
                ref field,
                ref message,
                ..
            } if field == "outposts.prices.value" && message.contains("must be >= 0")
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn load_aic_rejects_outpost_key_with_spaces() {
    make_guard!(guard);
    let catalog = load_catalog(None, guard).expect("load builtin catalog");
    let (_, price_item) = first_two_item_keys(&catalog);
    let src = format!(
        r#"
external_power_consumption_w = 0

[[outposts]]
key = " Camp "
money_cap_per_hour = 60
prices = {{ "{price_item}" = 1 }}
"#
    );

    let err = load_aic_from_str(&src, &catalog).expect_err("spaced outpost key should fail");
    assert_schema_location(&err, "aic.toml", "outposts.key", Some(0));
}

#[test]
fn load_aic_accepts_zero_external_power_when_other_fields_valid() {
    make_guard!(guard);
    let catalog = load_catalog(None, guard).expect("load builtin catalog");
    let (supply_item, price_item) = first_two_item_keys(&catalog);
    let src = format!(
        r#"
external_power_consumption_w = 0

[supply_per_min]
"{supply_item}" = 1

[[outposts]]
key = "Camp"
money_cap_per_hour = 60
prices = {{ "{price_item}" = 1 }}
"#
    );

    let aic = load_aic_from_str(&src, &catalog).expect("valid aic should load");
    assert_eq!(aic.external_power_consumption_w(), 0);
    assert_eq!(aic.outposts().len(), 1);
}

#[test]
fn default_aic_toml_roundtrip_is_loadable() {
    make_guard!(guard);
    let catalog = load_catalog(None, guard).expect("load builtin catalog");
    let src = default_aic_toml(&catalog).expect("build default aic toml");
    let loaded = load_aic_from_str(&src, &catalog).expect("default aic toml should load");

    assert!(
        !loaded.outposts().is_empty(),
        "default aic should have outposts"
    );
}
