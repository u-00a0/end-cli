use end_model::{Catalog, DisplayName, ItemDef, ItemId, Key, ThermalBankDef};
use generativity::make_guard;

fn key(value: &str) -> Key {
    value.try_into().unwrap()
}

fn name(value: &str) -> DisplayName {
    value.try_into().unwrap()
}

fn item_name<'id>(catalog: &Catalog<'id>, item: ItemId<'id>) -> String {
    catalog.item(item).unwrap().en.as_str().to_string()
}

fn with_catalog<R>(
    item_key: &'static str,
    bank_key: &'static str,
    f: impl for<'id> FnOnce(Catalog<'id>, ItemId<'id>) -> R,
) -> R {
    make_guard!(guard);
    let mut b = Catalog::builder(guard);
    let item = b
        .add_item(ItemDef {
            key: key(item_key),
            en: name("Ore"),
            zh: name("Ore"),
        })
        .unwrap();
    b.add_thermal_bank(ThermalBankDef {
        key: key(bank_key),
        en: name("Bank"),
        zh: name("Bank"),
    })
    .unwrap();
    let catalog = b.build().unwrap();
    f(catalog, item)
}

fn main() {
    with_catalog("ore-1", "bank-1", |catalog_1, ore_1| {
        with_catalog("ore-2", "bank-2", |catalog_2, _| {
            let _ = catalog_1;
            let _ = item_name(&catalog_2, ore_1);
        });
    });
}
