use std::num::NonZeroU32;

use crate::{DisplayName, Key};

use super::*;

fn key(value: &str) -> Key {
    value.try_into().expect("valid key")
}

fn name(value: &str) -> DisplayName {
    value.try_into().expect("valid display name")
}

fn sample_builder() -> (CatalogBuilder, ItemId, ItemId, FacilityId, FacilityId) {
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
            kind: FacilityKind::Machine,
            power_w: Some(NonZeroU32::new(10).expect("non-zero")),
            en: name("M1"),
            zh: name("M1"),
        })
        .expect("add machine");
    let thermal = builder
        .add_facility(FacilityDef {
            key: key("Thermal Bank"),
            kind: FacilityKind::ThermalBank,
            power_w: None,
            en: name("Thermal Bank"),
            zh: name("Thermal Bank"),
        })
        .expect("add thermal");
    (builder, a, b, machine, thermal)
}

#[test]
fn push_recipe_rejects_duplicate_items_in_same_list() {
    let (mut builder, a, b, machine, _) = sample_builder();
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
fn push_recipe_rejects_thermal_bank_facility() {
    let (mut builder, a, b, _, thermal) = sample_builder();
    let err = builder
        .push_recipe(
            thermal,
            60,
            vec![Stack { item: a, count: 1 }],
            vec![Stack { item: b, count: 1 }],
        )
        .expect_err("thermal bank should not be accepted as recipe facility");

    assert!(
        matches!(
            err,
            CatalogBuildError::RecipeFacilityMustBeMachine {
                facility_id,
                kind: FacilityKind::ThermalBank
            } if facility_id == thermal.as_u32()
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

    let (mut builder, _a, b, machine, _) = sample_builder();
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
fn add_facility_rejects_machine_without_power() {
    let mut builder = Catalog::builder();
    let err = builder
        .add_facility(FacilityDef {
            key: key("M2"),
            kind: FacilityKind::Machine,
            power_w: None,
            en: name("M2"),
            zh: name("M2"),
        })
        .expect_err("machine without power should fail");

    assert!(
        matches!(
            err,
            CatalogBuildError::MachineFacilityMissingPower { ref key } if key.as_str() == "M2"
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn add_facility_rejects_thermal_bank_with_power() {
    let mut builder = Catalog::builder();
    let err = builder
        .add_facility(FacilityDef {
            key: key("TB"),
            kind: FacilityKind::ThermalBank,
            power_w: Some(NonZeroU32::new(1).expect("non-zero")),
            en: name("TB"),
            zh: name("TB"),
        })
        .expect_err("thermal bank with power should fail");

    assert!(
        matches!(
            err,
            CatalogBuildError::ThermalBankFacilityHasPower { ref key } if key.as_str() == "TB"
        ),
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

    let (mut builder, _, _, _, _) = sample_builder();
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
    let (mut builder, a, _, _, _) = sample_builder();

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
