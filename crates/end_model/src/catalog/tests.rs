use std::num::NonZeroU32;

use crate::{DisplayName, Key};

use super::*;

fn key(value: &str) -> Key {
    value.try_into().expect("valid key")
}

fn name(value: &str) -> DisplayName {
    value.try_into().expect("valid display name")
}

fn nz(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("non-zero")
}

fn sample_builder() -> (CatalogBuilder, ItemId, ItemId, FacilityId) {
    let mut builder = Catalog::builder();
    let a = builder
        .add_item(ItemDef {
            key: key("A"),
            en: name("A"),
            zh: name("A"),
        })
        .expect("add item A");
    let b = builder
        .add_item(ItemDef {
            key: key("B"),
            en: name("B"),
            zh: name("B"),
        })
        .expect("add item B");
    let machine = builder
        .add_facility(FacilityDef {
            key: key("M1"),
            power_w: nz(10),
            en: name("M1"),
            zh: name("M1"),
        })
        .expect("add machine");
    builder
        .add_thermal_bank(ThermalBankDef {
            key: key("Thermal Bank"),
            en: name("Thermal Bank"),
            zh: name("Thermal Bank"),
        })
        .expect("add thermal bank");
    (builder, a, b, machine)
}

#[test]
fn push_recipe_rejects_duplicate_items_in_same_list() {
    let (mut builder, a, b, machine) = sample_builder();
    let err = builder
        .push_recipe(
            machine,
            60,
            vec![Stack { item: a, count: 1 }, Stack { item: a, count: 2 }],
            vec![Stack { item: b, count: 1 }],
        )
        .expect_err("duplicate ingredients should fail");

    assert!(
        matches!(
            err,
            CatalogBuildError::DuplicateRecipeItem {
                list: "ingredients",
                item_id
            } if item_id == a.as_u32()
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn push_recipe_rejects_out_of_catalog_facility_id() {
    let mut other = Catalog::builder();
    let _ = other
        .add_facility(FacilityDef {
            key: key("X1"),
            power_w: nz(10),
            en: name("X1"),
            zh: name("X1"),
        })
        .expect("add facility X1");
    let foreign_facility = other
        .add_facility(FacilityDef {
            key: key("X2"),
            power_w: nz(20),
            en: name("X2"),
            zh: name("X2"),
        })
        .expect("add facility X2");

    let (mut builder, a, b, _machine) = sample_builder();
    let err = builder
        .push_recipe(
            foreign_facility,
            60,
            vec![Stack { item: a, count: 1 }],
            vec![Stack { item: b, count: 1 }],
        )
        .expect_err("foreign facility id should fail");

    assert!(
        matches!(
            err,
            CatalogBuildError::UnknownRecipeFacilityId(facility_id)
                if facility_id == foreign_facility.as_u32()
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn push_recipe_rejects_out_of_catalog_item_id() {
    let mut other = Catalog::builder();
    let _ = other
        .add_item(ItemDef {
            key: key("X"),
            en: name("X"),
            zh: name("X"),
        })
        .expect("add item X");
    let _ = other
        .add_item(ItemDef {
            key: key("Y"),
            en: name("Y"),
            zh: name("Y"),
        })
        .expect("add item Y");
    let foreign_item = other
        .add_item(ItemDef {
            key: key("Z"),
            en: name("Z"),
            zh: name("Z"),
        })
        .expect("add item Z");

    let (mut builder, _a, b, machine) = sample_builder();
    let err = builder
        .push_recipe(
            machine,
            60,
            vec![Stack {
                item: foreign_item,
                count: 1,
            }],
            vec![Stack { item: b, count: 1 }],
        )
        .expect_err("foreign item id should fail");

    assert!(
        matches!(
            err,
            CatalogBuildError::UnknownRecipeItemId {
                list: "ingredients",
                item_id
            } if item_id == foreign_item.as_u32()
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn add_facility_rejects_duplicate_key() {
    let mut builder = Catalog::builder();
    let _ = builder
        .add_facility(FacilityDef {
            key: key("M2"),
            power_w: nz(10),
            en: name("M2"),
            zh: name("M2"),
        })
        .expect("first facility should be accepted");
    let err = builder
        .add_facility(FacilityDef {
            key: key("M2"),
            power_w: nz(20),
            en: name("M2-dupe"),
            zh: name("M2-dupe"),
        })
        .expect_err("duplicate facility key should fail");

    assert!(
        matches!(err, CatalogBuildError::DuplicateFacilityKey(ref key) if key.as_str() == "M2"),
        "unexpected error: {err:?}"
    );
}

#[test]
fn add_thermal_bank_rejects_duplicate_machine_key() {
    let mut builder = Catalog::builder();
    let _ = builder
        .add_facility(FacilityDef {
            key: key("M2"),
            power_w: nz(10),
            en: name("M2"),
            zh: name("M2"),
        })
        .expect("machine should be accepted");
    let err = builder
        .add_thermal_bank(ThermalBankDef {
            key: key("M2"),
            en: name("TB"),
            zh: name("TB"),
        })
        .expect_err("thermal bank key colliding with machine should fail");

    assert!(
        matches!(err, CatalogBuildError::DuplicateFacilityKey(ref key) if key.as_str() == "M2"),
        "unexpected error: {err:?}"
    );
}

#[test]
fn add_thermal_bank_rejects_multiple_entries() {
    let mut builder = Catalog::builder();
    builder
        .add_thermal_bank(ThermalBankDef {
            key: key("TB1"),
            en: name("TB1"),
            zh: name("TB1"),
        })
        .expect("first thermal bank should be accepted");
    let err = builder
        .add_thermal_bank(ThermalBankDef {
            key: key("TB2"),
            en: name("TB2"),
            zh: name("TB2"),
        })
        .expect_err("second thermal bank should fail");

    assert!(
        matches!(err, CatalogBuildError::MultipleThermalBanks),
        "unexpected error: {err:?}"
    );
}

#[test]
fn build_rejects_missing_thermal_bank() {
    let mut builder = Catalog::builder();
    let _ = builder
        .add_item(ItemDef {
            key: key("A"),
            en: name("A"),
            zh: name("A"),
        })
        .expect("add item");
    let _ = builder
        .add_facility(FacilityDef {
            key: key("M1"),
            power_w: nz(10),
            en: name("M1"),
            zh: name("M1"),
        })
        .expect("add machine");

    let err = builder
        .build()
        .expect_err("missing thermal bank should fail");
    assert!(
        matches!(err, CatalogBuildError::MissingThermalBank),
        "unexpected error: {err:?}"
    );
}

#[test]
fn push_power_recipe_rejects_out_of_catalog_item_id() {
    let mut other = Catalog::builder();
    let _ = other
        .add_item(ItemDef {
            key: key("X"),
            en: name("X"),
            zh: name("X"),
        })
        .expect("add item X");
    let _ = other
        .add_item(ItemDef {
            key: key("Y"),
            en: name("Y"),
            zh: name("Y"),
        })
        .expect("add item Y");
    let foreign_item = other
        .add_item(ItemDef {
            key: key("Z"),
            en: name("Z"),
            zh: name("Z"),
        })
        .expect("add item Z");

    let (mut builder, _, _, _) = sample_builder();
    let err = builder
        .push_power_recipe(PowerRecipe {
            ingredient: Stack {
                item: foreign_item,
                count: 1,
            },
            power_w: 100,
            time_s: 10,
        })
        .expect_err("foreign ingredient item should fail");

    assert!(
        matches!(
            err,
            CatalogBuildError::UnknownPowerRecipeIngredientItemId(item_id)
                if item_id == foreign_item.as_u32()
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn push_power_recipe_rejects_zero_fields() {
    let (mut builder, a, _, _) = sample_builder();

    let err_count = builder
        .push_power_recipe(PowerRecipe {
            ingredient: Stack { item: a, count: 0 },
            power_w: 1,
            time_s: 1,
        })
        .expect_err("zero ingredient count should fail");
    assert!(
        matches!(
            err_count,
            CatalogBuildError::PowerRecipeIngredientCountMustBePositive { item_id }
                if item_id == a.as_u32()
        ),
        "unexpected error: {err_count:?}"
    );

    let err_power = builder
        .push_power_recipe(PowerRecipe {
            ingredient: Stack { item: a, count: 1 },
            power_w: 0,
            time_s: 1,
        })
        .expect_err("zero power should fail");
    assert!(
        matches!(err_power, CatalogBuildError::PowerRecipePowerMustBePositive),
        "unexpected error: {err_power:?}"
    );

    let err_time = builder
        .push_power_recipe(PowerRecipe {
            ingredient: Stack { item: a, count: 1 },
            power_w: 1,
            time_s: 0,
        })
        .expect_err("zero time should fail");
    assert!(
        matches!(err_time, CatalogBuildError::PowerRecipeTimeMustBePositive),
        "unexpected error: {err_time:?}"
    );
}
